use std::{mem::transmute_copy, time::Duration};

use button_input::ButtonInput;
use rotary_input::{RotaryInput, RotaryInputTarget};
use serialport::SerialPort;
use uinput::{event::{absolute, controller}, Event};

pub mod button_input;
pub mod rotary_input;

fn main() {
    // create device
    let mut device = uinput::default().unwrap()
        .name("UInput Steering Wheel").unwrap()
        .bus(0x0003).vendor(0x046D)
        .product(0xC29B).version(0x0110)
        .event(absolute::Position::X).unwrap()
        .min(-900).max(900)
        .fuzz(0).flat(0)
        .event(absolute::Position::Y).unwrap()
        .min(0).max(255)
        .fuzz(0).flat(0)
        .event(absolute::Position::Z).unwrap()
        .min(0).max(255)
        .fuzz(0).flat(0)
        .event(absolute::Position::RX).unwrap()
        .min(0).max(255)
        .fuzz(0).flat(0)
        .event(absolute::Wheel::Position).unwrap()
        .min(-900).max(900)
        .fuzz(0).flat(0)
        .event(controller::GamePad::TR).unwrap()
        .event(controller::GamePad::TR2).unwrap()
        .event(controller::GamePad::TL).unwrap()
        .event(controller::GamePad::TL2).unwrap()
        .event(controller::GamePad::North).unwrap()
        .event(controller::GamePad::South).unwrap()
        .event(controller::GamePad::East).unwrap()
        .event(controller::GamePad::West).unwrap()
        .event(controller::GamePad::A).unwrap()
        .event(controller::GamePad::B).unwrap()
        .create().unwrap();
    device.synchronize().unwrap();

    // get an active USB port
    let port_info = serialport::available_ports().expect("No ports found!")
        .into_iter()
        .filter(|port| port.port_name.contains("USB"))
        .next();
    if port_info.is_none() { println!("No active USB port found!") }
    let port_info = port_info.unwrap();

    // open port
    let mut port = serialport::new(port_info.port_name, 115200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port");

    // wait for aduino to initialize
    std::thread::sleep(Duration::from_secs(2));

    // send a bunch of inputs to prep uinput (IDK why I need to do this)
    let _ = device.send(absolute::Wheel::Gas, 127);
    let _ = device.send(absolute::Wheel::Gas, 0);
    let _ = device.send(absolute::Wheel::Brake, 127);
    let _ = device.send(absolute::Wheel::Brake, 0);
    device.synchronize().unwrap();
    let _ = device.send(Event::Controller(controller::Controller::DPad(controller::DPad::Up)), 0);
    let _ = device.send(Event::Controller(controller::Controller::DPad(controller::DPad::Down)), 0);
    let _ = device.send(Event::Controller(controller::Controller::DPad(controller::DPad::Left)), 0);
    let _ = device.send(Event::Controller(controller::Controller::DPad(controller::DPad::Right)), 0);
    device.synchronize().unwrap();

    // build encoder inputs
    let mut encoder_inputs: [RotaryInput; 4] = [
        RotaryInput::new(RotaryInputTarget::Button { button: controller::GamePad::TL, cross_value: 0.25 }, 0.0),
        RotaryInput::new(RotaryInputTarget::Button { button: controller::GamePad::TR, cross_value: 0.25 }, 0.0),
        RotaryInput::new(RotaryInputTarget::Axis { axis: absolute::Position::Z.into(), flip: false }, 0.0),
        RotaryInput::new(RotaryInputTarget::Axis { axis: absolute::Position::Y.into(), flip: true }, 0.0),
    ];

    // build buttons
    let mut button_inputs: [ButtonInput; 6] = [ButtonInput::default(); 6];
    let button_targets: [Event; 6] = [
        Event::Controller(controller::Controller::GamePad(controller::GamePad::North)),
        Event::Controller(controller::Controller::GamePad(controller::GamePad::South)),
        Event::Controller(controller::Controller::GamePad(controller::GamePad::East)),
        Event::Controller(controller::Controller::GamePad(controller::GamePad::West)),
        Event::Controller(controller::Controller::GamePad(controller::GamePad::A)),
        Event::Controller(controller::Controller::GamePad(controller::GamePad::B)),
    ];

    let mut wheel_offset = 0.0;

    loop {
        let _ = port.write(&[0x00]);
        
        // wait until we have something to read
        loop {
            let byte = read::<u8, 1>(&mut port);
            if byte == 255 { break }
        }

        print!("\r");

        // read headers
        let tick_length = read::<u32, 4>(&mut port);
        let longest_length = read::<u32, 4>(&mut port);
        print!("{:0.5} | {:05} | {:05}", 3, tick_length, longest_length);

        // read inputs
        let angle = read::<f32, 4>(&mut port);

        print!(" | {:>5.1}", angle - wheel_offset);

        // read encoders
        let num_encoders = read::<u32, 4>(&mut port);
        print!(" | {:02}", num_encoders);
        (0 .. num_encoders).for_each(|idx| {
            let position = read::<i16, 2>(&mut port);
            encoder_inputs[idx as usize].update(&mut device, position);
            print!(" | {:02}", position);
        });

        // read buttons
        let num_buttons = read::<u16, 2>(&mut port);
        read::<u16, 2>(&mut port);
        print!(" | {:02}", num_buttons);
        (0 .. num_buttons).for_each(|idx| {
            let input = read::<u8, 1>(&mut port);
            button_inputs[idx as usize].update(input > 0);
            if idx == 1 {
                if button_inputs[idx as usize].was_released() {
                    wheel_offset = angle;
                }
            } else {
                if button_inputs[idx as usize].was_pressed() || button_inputs[idx as usize].was_released() {
                    let _ = device.send(
                        button_targets[idx as usize], 
                        if button_inputs[idx as usize].was_pressed() { 255 } else { 0 }
                    );
                }
            }
            print!(" | {:01}", input);
        });

        // write steering input
        device.position(
            &absolute::Position::X, 
            ((angle - wheel_offset) * 7.0) as i32
        ).unwrap();

        // sync input device
        device.synchronize().unwrap();

        // slow down
        std::thread::sleep(Duration::from_millis(4));
    }
}

fn read<O, const LEN: usize>(port: &mut Box<dyn SerialPort>) -> O {
    assert_eq!(size_of::<O>(), LEN, "Size mismatch in read!");
    unsafe { transmute_copy(&read_bytes::<LEN>(port)) }
}

fn read_bytes<const LEN: usize>(port: &mut Box<dyn SerialPort>) -> [u8; LEN] {
    let mut bytes = [0u8; LEN];
    let _ = port.read(&mut bytes);
    return bytes;
}
