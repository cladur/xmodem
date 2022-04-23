mod common;
mod receiver;
mod transmitter;

use receiver::{new_receive};
use transmitter::{transmit};

use std::io;
use std::io::prelude::*;


fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

pub enum Receiver_Mode {
    Normal,
    CRC,
}

fn main() {
    /*
    println!("1. Checksum");
    println!("2. CRC");
    std::io::stdin().read_line(&mut input).unwrap();
    let variant = input.trim().parse::<u32>().unwrap();
    */
    let mut input = String::new();
    println!("1. Transmit data");
    println!("2. Receive data");
    // Get input
    std::io::stdin().read_line(&mut input).unwrap();
    let number = input.trim().parse::<u32>().unwrap();
    match number {
        1 => {
            println!("Transmitting data...");
            println!("Opening port COM1...");
            let mut port = serialport::new("COM1", 115_200).open().unwrap();
            let data = "This Message is exactly 128 byteThis Message is exactly 128 byteThis Message is exactly 128 byteThis Message is exactly 128 byte".as_bytes();
            //This Message is exactly 128 byteThis Message is exactly 128 byteThis Message is exactly 128 byteThis Message is exactly 128 byte
            transmit(&mut port, &data);
        }
        2 => {
            println!("Receiving data...");
            println!("Opening port COM2...");
            let mut port = serialport::new("COM2", 115_200).open().unwrap();
            let data = new_receive(&mut port, Receiver_Mode::CRC);
            println!(
                "Received data: {:?}",
                String::from_utf8_lossy(data.as_slice())
            );
        }
        _ => {
            println!("Invalid input");
        }
    }
}
