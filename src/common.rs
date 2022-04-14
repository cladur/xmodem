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
