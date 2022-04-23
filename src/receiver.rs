use crate::{common::{crc, Symbol}, ReceiverMode};
use serialport::SerialPort;

pub fn alg_checksum(bytes: &[u8]) -> u8 {
    let mut checksum: u16 = 0;
    for byte in bytes {
        checksum += *byte as u16;
        checksum %= 256;
    }
    checksum as u8
}

pub fn new_receive(port: &mut Box<dyn SerialPort>, mode: ReceiverMode) -> Vec<u8> {
    let mut char_byte = [0u8; 1];
    let mut recieved_bytes = [0u8; 128];
    let mut all_recieved_bytes: Vec<u8> = Vec::new();
    match mode {
        ReceiverMode::Normal => {
            char_byte[0] = Symbol::NAK as u8;
        }
        ReceiverMode::CRC => {
            char_byte[0] = Symbol::C as u8;
        }
    }

    while port.bytes_to_read().unwrap() == 0 {
        port.write(&char_byte).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let mut char = ' ';

    while char != Symbol::EOT as u8 as char {
        std::thread::sleep(std::time::Duration::from_millis(100));
        port.read_exact(&mut char_byte).unwrap();
        char = char_byte[0] as char;
        println!("recieved: {}", char);

        if char_byte[0] == Symbol::EOT as u8 {
            char_byte[0] = Symbol::ACK as u8;
            port.write(&char_byte).unwrap();
            break;
        }

        if char_byte[0] == Symbol::SOH as u8 {
            // get packet number
            port.read_exact(&mut char_byte).unwrap();
            let packet_num = char_byte[0];
            // get packet number complement
            port.read_exact(&mut char_byte).unwrap();
            let packet_complement = char_byte[0];
            println!("packet_num: {}", packet_num);
            port.read_exact(&mut recieved_bytes).unwrap();

            match mode {
                ReceiverMode::Normal => {
                    let mut checksum = [0u8; 1];
                    port.read_exact(&mut checksum).unwrap();

                    // calculate checksum for recieved bytes
                    let checksum_calc = alg_checksum(&recieved_bytes);
                    let mut packet = [0u8; 128];
                    packet[..128].clone_from_slice(&recieved_bytes);

                    // compare checksums and packet numbers/fills
                    if (checksum_calc == checksum[0]) && (packet_complement == 255 - packet_num) {
                        all_recieved_bytes.extend_from_slice(&packet);
                        char_byte[0] = Symbol::ACK as u8;
                        port.write(&char_byte).unwrap();
                    } else {
                        char_byte[0] = Symbol::NAK as u8;
                        port.write(&char_byte).unwrap();
                    }
                }
                ReceiverMode::CRC => {
                    // read 2 bytes, checksum is 16 bits long
                    let mut checksum = [0u8; 2];
                    port.read_exact(&mut checksum).unwrap();

                    let mut packet = [0u8; 128];
                    packet[..128].clone_from_slice(&recieved_bytes);

                    // calculate checksum for received bytes
                    let checksum_calc = crc(&packet);
                    let packet_checksum: u16 = (checksum[0] as u16) << 8 | checksum[1] as u16;
                    if checksum_calc == packet_checksum {
                        all_recieved_bytes.extend_from_slice(&packet);
                        char_byte[0] = Symbol::ACK as u8;
                        port.write(&char_byte).unwrap();
                    } else {
                        char_byte[0] = Symbol::NAK as u8;
                        port.write(&char_byte).unwrap();
                    }
                }
            }
        }
    }

    all_recieved_bytes
}
