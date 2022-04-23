use crate::{common::{crc, Symbol}, Receiver_Mode};
use serialport::SerialPort;

pub fn alg_checksum(bytes: &[u8]) -> u8 {
    let mut checksum: u16 = 0;
    for byte in bytes {
        checksum += *byte as u16;
        checksum %= 256;
    }
    checksum as u8
}

pub fn new_receive(port: &mut Box<dyn SerialPort>, mode: Receiver_Mode) -> Vec<u8> {
    let mut char_byte = [0u8; 1];
    let mut recieved_bytes = [0u8; 128];
    let mut all_recieved_bytes: Vec<u8> = Vec::new();
    match mode {
        Receiver_Mode::Normal => {
            char_byte[0] = Symbol::NAK as u8;
        }
        Receiver_Mode::CRC => {
            char_byte[0] = Symbol::C as u8;
        }
    }

    while port.bytes_to_read().unwrap() == 0 {
        port.write(&char_byte);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let mut char = ' ';

    // while (znak != EOT)
    while char != Symbol::EOT as u8 as char {
        std::thread::sleep(std::time::Duration::from_millis(100));
        port.read_exact(&mut char_byte);
        char = char_byte[0] as char;
        println!("recieved: {}", char);

        // if (znakByte[0] == EOT)
        if char_byte[0] == Symbol::EOT as u8 {
            char_byte[0] = Symbol::ACK as u8;
            port.write(&char_byte);
            break;
        }

        // if (znakByte[0] == SOH)
        if char_byte[0] == Symbol::SOH as u8 {
            let mut packet_num: u8 = 0;
            let mut packet_fill: u8 = 0;
            port.read_exact(&mut char_byte);
            packet_num = char_byte[0];
            port.read_exact(&mut char_byte);
            packet_fill = char_byte[0];
            println!("packet_num: {}", packet_num);
            port.read_exact(&mut recieved_bytes);

            // if (mode == NAK)
            match mode {
                Receiver_Mode::Normal => {
                    let mut checksum = [0u8; 1];
                    port.read_exact(&mut checksum);

                    // calculate checksum for recieved bytes
                    let mut checksum_calc = alg_checksum(&recieved_bytes);
                    let mut packet = [0u8; 128];
                    packet[..128].clone_from_slice(&recieved_bytes);

                    // compare checksums and packet numbers/fills
                    if (checksum_calc == checksum[0]) && (packet_fill == 255 - packet_num) {
                        all_recieved_bytes.extend_from_slice(&packet);
                        char_byte[0] = Symbol::ACK as u8;
                        port.write(&char_byte);
                    } else {
                        char_byte[0] = Symbol::NAK as u8;
                        port.write(&char_byte);
                    }
                }
                Receiver_Mode::CRC => {
                    // read 2 bytes, checksum is 16 bits long
                    let mut checksum = [0u8; 2];
                    port.read_exact(&mut checksum);

                    let mut packet = [0u8; 128];
                    packet[..128].clone_from_slice(&recieved_bytes);

                    // calculate checksum for received bytes
                    let checksum_calc = crc(&packet);
                    let packet_checksum: u16 = (checksum[0] as u16) << 8 | checksum[1] as u16;
                    if checksum_calc == packet_checksum {
                        all_recieved_bytes.extend_from_slice(&packet);
                        char_byte[0] = Symbol::ACK as u8;
                        port.write(&char_byte);
                    } else {
                        char_byte[0] = Symbol::NAK as u8;
                        port.write(&char_byte);
                    }
                }
            }
        }
    }

    all_recieved_bytes
}

pub fn receive(port: &mut Box<dyn SerialPort>) -> Vec<u8> {
    let mut buf = [0u8; 1024];
    let mut block_size = 0;
    // Send NAK every 10 secounds for 1 minute
    //for _ in 0..6 {
        while port.bytes_to_read().unwrap() == 0 {
            println!("Sending NAK...");
            port.write(&[Symbol::NAK as u8]).unwrap();
        }
        block_size = port.read(&mut buf).unwrap();


    //}
    // LOOP Get block, check if it is correct, send ACK or NAK
    let mut first_block = false;
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
