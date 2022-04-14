use crate::common::{crc, Symbol};
use serialport::SerialPort;

pub fn receive(port: &mut Box<dyn SerialPort>) -> Vec<u8> {
    let mut buf = [0u8; 1024];
    let mut block_size = 0;
    // Send NAK every 10 secounds for 1 minute
    for _ in 0..6 {
        println!("Sending NAK...");
        port.write(&[Symbol::NAK as u8]).unwrap();
        block_size = port.read(&mut buf).unwrap();
        if buf[0] == Symbol::SOH as u8 {
            break;
        }
        println!("Sleeping...");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    // LOOP Get block, check if it is correct, send ACK or NAK
    let mut first_block = true;
    let mut data = Vec::new();
    loop {
        if !first_block {
            block_size = port.read(&mut buf).unwrap();
        }

        // If we get EOT, send ACK
        if buf[0] == Symbol::EOT as u8 {
            println!("Received EOT...");
            println!("Sending ACK...");
            port.write(&[Symbol::ACK as u8]).unwrap();
            break;
        }

        first_block = false;
        let block_number = buf[1];
        let inverse_block_number = buf[2];
        println!("Receiving block {}... ({} bytes)", block_number, block_size);

        // Check if block numbers are correct
        if 255 - inverse_block_number != block_number {
            println!("Block {} is invalid... (Bad block number)", block_number);
            println!("Sending NAK...");
            port.write(&[Symbol::NAK as u8]).unwrap();
            continue;
        }

        // Check if data checksum is correct
        let checksum = buf[131];
        let actual_checksum = buf[3..131]
            .iter()
            .fold(0, |acc, &x| u8::wrapping_add(acc, x));

        if checksum != actual_checksum {
            println!("Block {} is invalid... (Bad checksum)", block_number);
            println!("Sending NAK...");
            port.write(&[Symbol::NAK as u8]).unwrap();
            continue;
        }

        data.extend(&buf[3..132]);

        port.write(&[Symbol::ACK as u8]).unwrap();
    }
    data
}

pub fn receive_crc(port: &mut Box<dyn SerialPort>) -> Vec<u8> {
    let mut buf = [0u8; 1024];
    let mut block_size = 0;
    // Send NAK every 10 secounds for 1 minute
    for _ in 0..6 {
        println!("Sending C...");
        port.write(&[Symbol::C as u8]).unwrap();
        block_size = port.read(&mut buf).unwrap();
        if buf[0] == Symbol::SOH as u8 {
            break;
        }
        println!("Sleeping...");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    // LOOP Get block, check if it is correct, send ACK or NAK
    let mut first_block = true;
    let mut data = Vec::new();
    loop {
        if !first_block {
            block_size = port.read(&mut buf).unwrap();
        }
        first_block = false;

        // If we get EOT, send ACK
        if buf[0] == Symbol::EOT as u8 {
            println!("Received EOT...");
            println!("Sending ACK...");
            port.write(&[Symbol::ACK as u8]).unwrap();
            break;
        }

        let block_number = buf[1];
        let inverse_block_number = buf[2];
        println!("Receiving block {}... ({} bytes)", block_number, block_size);

        // Check if block numbers are correct
        if 255 - inverse_block_number != block_number {
            println!("Block {} is invalid... (Bad block number)", block_number);
            println!("Sending NAK...");
            port.write(&[Symbol::NAK as u8]).unwrap();
            continue;
        }

        // Check if data checksum is correct
        let checksum_high = buf[131];
        let checksum_low = buf[132];
        let checksum = (checksum_high as u16) << 8 | checksum_low as u16;
        let actual_checksum = crc(&buf[3..131]);

        if checksum != actual_checksum {
            println!("Block {} is invalid... (Bad checksum)", block_number);
            println!("Sending NAK...");
            port.write(&[Symbol::NAK as u8]).unwrap();
            continue;
        }

        data.extend(&buf[3..132]);

        port.write(&[Symbol::ACK as u8]).unwrap();
    }
    data
}
