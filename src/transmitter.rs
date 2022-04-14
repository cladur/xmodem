use crate::common::{crc, Symbol};
use serialport::SerialPort;

pub fn transmit(port: &mut Box<dyn SerialPort>, data: &[u8]) {
    // Wait for NAK from receiver
    loop {
        let mut buf = [0u8; 1];
        port.read(&mut buf).unwrap();
        if buf[0] == Symbol::NAK as u8 {
            println!("Received NAK...");
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    for (i, block) in data.chunks(128).enumerate() {
        loop {
            println!("Sending block {}", i);
            let mut buf: Vec<u8> = Vec::with_capacity(132);
            buf.push(Symbol::SOH as u8);
            buf.push(i as u8);
            buf.push(255 - i as u8);
            buf.extend_from_slice(block);
            if block.len() != 128 {
                for _ in 0..(128 - block.len()) {
                    buf.push(0);
                }
            }
            let checksum = buf.iter().fold(0, |acc, &x| u8::wrapping_add(acc, x));
            buf.push(checksum);
            port.write(&buf);
            // Receive ACK or NAK
            let mut buf = [0u8; 1];
            port.read(&mut buf).unwrap();
            if buf[0] == Symbol::ACK as u8 {
                println!("Received ACK...");
                break;
            } else if buf[0] == Symbol::NAK as u8 {
                println!("Received NAK...");
            }
        }
    }

    // Send EOT
    println!("Sending EOT...");
    port.write(&[Symbol::EOT as u8]);
    loop {
        let mut buf = [0u8; 1];
        port.read(&mut buf).unwrap();
        if buf[0] == Symbol::ACK as u8 {
            println!("Received ACK...");
            break;
        }
    }
}

pub fn transmit_crc(port: &mut Box<dyn SerialPort>, data: &[u8]) {
    // Wait for 'C' from receiver
    loop {
        let mut buf = [0u8; 1];
        port.read(&mut buf).unwrap();
        if buf[0] == Symbol::C as u8 {
            println!("Received C...");
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    for (i, block) in data.chunks(128).enumerate() {
        loop {
            println!("Sending block {}", i);
            let mut buf: Vec<u8> = Vec::with_capacity(133);
            buf.push(Symbol::SOH as u8);
            buf.push(i as u8);
            buf.push(255 - i as u8);
            buf.extend_from_slice(block);
            if block.len() != 128 {
                for _ in 0..(128 - block.len()) {
                    buf.push(0);
                }
            }
            let checksum = crc(&buf[3..]);
            buf.extend(checksum.to_be_bytes());
            port.write(&buf);
            // Receive ACK or NAK
            let mut buf = [0u8; 1];
            port.read(&mut buf).unwrap();
            if buf[0] == Symbol::ACK as u8 {
                println!("Received ACK...");
                break;
            } else if buf[0] == Symbol::NAK as u8 {
                println!("Received NAK...");
            }
        }
    }

    // Send EOT
    println!("Sending EOT...");
    port.write(&[Symbol::EOT as u8]);
    loop {
        let mut buf = [0u8; 1];
        port.read(&mut buf).unwrap();
        if buf[0] == Symbol::ACK as u8 {
            println!("Received ACK...");
            break;
        }
    }
}
