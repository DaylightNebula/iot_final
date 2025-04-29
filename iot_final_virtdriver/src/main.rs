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
    let mut port = serialport::new(port_info.port_name, 9600)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port");

    for _ in 0 .. 4 {
        let _ = port.write(&[0x00]);
        let _ = port.flush();
        std::thread::sleep(Duration::from_millis(10));
    }

    loop {
        let _ = port.write(&[0x00]);
        let _ = port.flush();

        print!("\r");

        // wait until we have something to read
        while port.bytes_to_read().unwrap_or(0) < 12 {}
        let total_bytes = port.bytes_to_read().unwrap_or(0);

        // read headers
        let tick_length = read::<u32, 4>(&mut port);
        let longest_length = read::<u32, 4>(&mut port);
        print!("{:0.5} | {:05} | {:05}", total_bytes, tick_length, longest_length);

        // read inputs
        let angle = read::<f32, 4>(&mut port);

        println!(" | {:>5.1}", angle);

        // read encoders
        // let num_encoders: i32 = unsafe { std::mem::transmute(read_bytes::<4>(&mut port)) };
        // if num_encoders > 4 { continue }
        // let mut encoders = Vec::<i16>::with_capacity(num_encoders as usize);
        // for _ in 0 .. num_encoders {
        //     let rotation: i16 = unsafe { std::mem::transmute(read_bytes::<2>(&mut port)) };
        //     encoders.push(rotation);
        // }

        // println!("Angle {angle_z}, Encoders {encoders:?}");

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
