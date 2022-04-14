mod common;
mod receiver;
mod transmitter;

use receiver::{receive, receive_crc};
use transmitter::{transmit, transmit_crc};

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
            let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam lectus odio, cursus ut elit et, vulputate consectetur nulla. Quisque erat nibh, gravida ac nunc eget, pellentesque tincidunt mauris. Morbi semper mattis facilisis. Pellentesque ac ipsum ut arcu mollis viverra in id erat. Vivamus cras amet.".as_bytes();
            transmit(&mut port, &data);
        }
        2 => {
            println!("Receiving data...");
            println!("Opening port COM2...");
            let mut port = serialport::new("COM2", 115_200).open().unwrap();
            let data = receive(&mut port);
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
