use std::time::Duration;

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

    loop {
        let _ = port.write(&[0x00]);

        // wait until we have something to read
        while port.bytes_to_read().unwrap_or(0) < 1 {}

        // read angle
        let angle_z: f32 = unsafe { std::mem::transmute(read_bytes::<4>(&mut port)) };

        // read encoders
        let num_encoders: i32 = unsafe { std::mem::transmute(read_bytes::<4>(&mut port)) };
        if num_encoders > 4 { continue }
        let mut encoders = Vec::<i16>::with_capacity(num_encoders as usize);
        for _ in 0 .. num_encoders {
            let rotation: i16 = unsafe { std::mem::transmute(read_bytes::<2>(&mut port)) };
            encoders.push(rotation);
        }

        println!("Angle {angle_z}, Encoders {encoders:?}");

        // slow down
        std::thread::sleep(Duration::from_millis(4));
    }
}

fn read_bytes<const LEN: usize>(port: &mut Box<dyn SerialPort>) -> [u8; LEN] {
    let mut bytes = [0u8; LEN];
    let _ = port.read(&mut bytes);
    return bytes;
}
