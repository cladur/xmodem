use crate::{
    common::{crc, Symbol},
    receiver::alg_checksum,
};
use serialport::SerialPort;

pub fn transmit(port: &mut Box<dyn SerialPort>, data: &[u8]) {
    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut mode = Symbol::NAK as u8;

    // utwórz 128 bajtowe pakiety danych do wysłania i ewentualnie dopełnij je zerami
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

        // dla pierwszego pakietu sprawdź tryb sumy kontrolnej (NAK lub CRC)
        while (char != Symbol::NAK as u8 as char)
            && (char != Symbol::C as u8 as char)
            && !connection
        {
            if port.bytes_to_read().unwrap() != 0 {
                // Odczytaj jeden bajt z portu (powinno być NAK lub C)
                port.read_exact(&mut char_byte).unwrap();
                println!("Received: {}", char_byte[0] as char);
            }
            char = char_byte[0] as char;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Jeżeli doszliśmy tutaj, oznacza to że pierwszy bajt został odebrany
        // ustawiamy zmienną connection na true, żeby nie sprawdzać pierwszego bajtu ponownie.
        if !connection {
            connection = true;
            mode = char_byte[0];
        }

        // przygotowanie nagłówka
        let mut header = [0u8; 3];
        header[0] = Symbol::SOH as u8;
        // nr. pakietu
        header[1] = i as u8;
        // dopełnienie nr. pakietu
        header[2] = 255 - i as u8;

        // Pętla wysyłająca pojedyńcze bloki
        loop {
            // wysłanie nagłówka
            port.write(&header).unwrap();
            println!(
                // wypisanie danych nagłówka na konsoli
                "Send SOH\nSend Packet no.: {}\nSend Packet complement: {}",
                i,
                255 - i
            );
            //wysłanie jednego pakietu danych
            port.write(&packet).unwrap();

            // choose the checksum algorithm
            if mode == Symbol::NAK as u8 {
                // Jeżeli tryb sumy kontrolnej to NAK, wysyłamy pakiet (1 bajt) ze zwykłą algebraiczną sumą kontrolną.
                let checksum = alg_checksum(&packet);
                // wysyłamy bajt symy kontrolnej
                port.write(&[checksum]).unwrap();
            } else if mode == Symbol::C as u8 {
                // Jeżeli tryb sumy kontrolnej to CRC, wysyłamy pakiet (2 bajty) z sumą kontrolną CRC.
                let crc_packet = crc(&packet);
                // Obliczamy sumę kontrolną CRC
                let mut checksum = [0u8; 2];
                // Rozdzielamy sumę kontrolną na 2 bajty
                checksum[0] = (crc_packet >> 8) as u8;
                checksum[1] = (crc_packet) as u8;
                // wyślij dwa bajty sumy kontrolnej
                port.write(&checksum).unwrap();
            }

            loop {
                // odczytujemy bajt z portu
                port.read_exact(&mut char_byte).unwrap();
                println!("Received: {}", char_byte[0] as char);
                // oczekujemy na odebranie ACK lub NAK
                if char_byte[0] != Symbol::ACK as u8 && char_byte[0] != Symbol::NAK as u8 {
                    continue;
                } else {
                    break;
                }
            }

            // Jeżeli odebrano ACK, wyjdź z pętli
            if char_byte[0] == Symbol::ACK as u8 {
                break;
            }
            // Jeżeli odebrano NAK, powtórz wysłanie tego samego bloku.
        }
    }

    // Jeżeli dotarliśmy tutaj, oznacza to, że wysłaliśmy wszystkie bloki.
    let mut char_byte = [Symbol::EOT as u8];
    // Wysyłamy EOT żeby zakończyć transmisję.
    port.write(&char_byte).unwrap();

    // Oczekujemy na odebranie ACK.
    while char_byte[0] != Symbol::ACK as u8 {
        if port.bytes_to_read().unwrap() != 0 {
            port.read_exact(&mut char_byte).unwrap();
            println!("Received: {}", char_byte[0] as char);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    // Jeżeli odebraliśmy ACK to transmisja zostaje zakończona.
    println!("TRANSMISSION FINISHED!")
}
