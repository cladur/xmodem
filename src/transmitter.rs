use crate::{
    common::{crc, Symbol},
    receiver::alg_checksum,
};
use serialport::SerialPort;

pub fn transmit(port: &mut Box<dyn SerialPort>, data: &[u8]) {
    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut mode = Symbol::NAK as u8;

    // split data in 128 byte packets
    for block in data.chunks(128) {
        let mut packet: Vec<u8> = vec![0; 128];
        packet.fill(0);
        for j in 0..block.len() {
            packet.remove(j);
            packet.insert(j, block[j]);
        }
        packets.push(packet);
    }

    let mut connection = false;
    for (i, packet) in packets.into_iter().enumerate() {
        let mut char = ' ';
        let mut char_byte = [0u8; 1];

        while (char != Symbol::NAK as u8 as char)
            && (char != Symbol::C as u8 as char)
            && !connection
        {
            if port.bytes_to_read().unwrap() != 0 {
                port.read_exact(&mut char_byte).unwrap();
                println!("Received: {}", char_byte[0] as char);
            }
            char = char_byte[0] as char;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if !connection {
            connection = true;
            mode = char_byte[0];
        }

        let mut header = [0u8; 3];
        header[0] = Symbol::SOH as u8;
        header[1] = i as u8;
        header[2] = 255 - i as u8;

        loop {
            port.write(&header).unwrap();
            println!(
                "Send SOH\nSend Packet no.: {}\nSend Packet complement: {}",
                i,
                255 - i
            );
            port.write(&packet).unwrap();

            // choose the checksum algorithm
            if mode == Symbol::NAK as u8 {
                let checksum = alg_checksum(&packet);
                port.write(&[checksum]).unwrap();
            } else if mode == Symbol::C as u8 {
                let crc_packet = crc(&packet);
                let mut checksum = [0u8; 2];
                checksum[0] = (crc_packet >> 8) as u8;
                checksum[1] = (crc_packet) as u8;
                port.write(&checksum).unwrap();
            }

            loop {
                port.read_exact(&mut char_byte).unwrap();
                println!("Received: {}", char_byte[0] as char);
                if char_byte[0] != Symbol::ACK as u8 && char_byte[0] != Symbol::NAK as u8 {
                    continue;
                } else {
                    break;
                }
            }

            if char_byte[0] == Symbol::ACK as u8 {
                break;
            }
        }
    }
    let mut char_byte = [Symbol::EOT as u8];
    port.write(&char_byte).unwrap();

    while char_byte[0] != Symbol::ACK as u8 {
        if port.bytes_to_read().unwrap() != 0 {
            port.read_exact(&mut char_byte).unwrap();
            println!("Received: {}", char_byte[0] as char);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("TRANSMISSION FINISHED!")
}
