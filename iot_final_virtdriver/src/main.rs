use std::{mem::transmute_copy, time::Duration};

use serialport::SerialPort;

fn main() {
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

        print!(" | {:>5.1}", angle);

        // read encoders
        let num_encoders = read::<u32, 4>(&mut port);
        print!(" | {:02}", num_encoders);
        (0 .. num_encoders).for_each(|_idx| {
            let position = read::<i16, 2>(&mut port);
            print!(" | {:02}", position);
        });

        // read buttons
        let num_buttons = read::<u16, 2>(&mut port);
        read::<u16, 2>(&mut port);
        print!(" | {:02}", num_buttons);
        (0 .. num_buttons).for_each(|_idx| {
            let input = read::<u8, 1>(&mut port);
            print!(" | {:01}", input);
        });

        println!();

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
