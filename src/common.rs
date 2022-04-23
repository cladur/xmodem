use std::{fs, io::Write};

pub enum Symbol {
    /// Start of Header
    SOH = 0x01,
    /// End of Transmission
    EOT = 0x04,
    /// Acknowledge
    ACK = 0x06,
    /// Not Acknowledge
    NAK = 0x15,
    /// End of Transmission Block
    ETB = 0x17,
    /// Cancel
    CAN = 0x18,
    /// ASCII 'C'
    C = 0x43,
}

pub fn crc(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for byte in data {
        crc = crc ^ (*byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = crc << 1 ^ 0x1021;
            } else {
                crc = crc << 1;
            }
        }
    }
    crc
}

pub fn file_to_u8(file: &str) -> Vec<u8> {
    let content = fs::read_to_string(file).unwrap();
    let mut vec: Vec<u8> = Vec::new();
    vec = content.bytes().collect();
    vec
}

pub fn u8_to_file(file: &str, data: &[u8]) {
    let mut file = fs::File::create(file).unwrap();
    file.write_all(data);
}
