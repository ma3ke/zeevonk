use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;

use crate::data::Data;

/// Listens for new websocket connections on `address` and spawns a new thread to listen on for
/// each connection. When a connection receives data, it is sent over the `sender` channel handler.
pub fn listener(address: &str, sender: Sender<Data>) {
    if let Ok(listener) = TcpListener::bind(address) {
        println!("Listening on: {address} ...\n");
        for client in listener.incoming() {
            if let Ok(client) = client {
                let sender = sender.clone();

                handle_client(client, sender);
            } else {
                eprintln!("Invalid client!")
            }
        }
    } else {
        eprintln!("Failed to start listening on {address}.");
    }
}

fn parse_led_count_bytes(bytes: [u8; 2]) -> u16 {
    u16::from_be_bytes(bytes)
}

fn handle_client(mut client: TcpStream, sender: Sender<Data>) {
    std::thread::spawn(move || loop {
        // Get the number led bytes that are in this message.
        let mut led_count_bytes = [0u8; 2];
        if client.read_exact(&mut led_count_bytes).is_ok() {
            let led_count = parse_led_count_bytes(led_count_bytes);

            // Get the led bytes.
            let mut led_bytes = vec![0u8; (led_count * 3) as usize];
            if client.read_exact(&mut led_bytes).is_ok() {
                if let Ok(data) = Data::from_bytes_vec(led_bytes) {
                    // Send the data over the channel.
                    sender.send(data).expect("channel send error");
                } else {
                    panic!("Failed to get create data from bytes");
                }
            } else {
                break;
            }
        } else {
            break;
        }
    });
}

mod tests {
    #[test]
    fn parsing_led_count_bytes_no_overflow() {
        use crate::listener::parse_led_count_bytes;

        let bytes = [0, 42];
        let parsed_value = parse_led_count_bytes(bytes);
        assert_eq!(parsed_value, 42)
    }

    #[test]
    fn parsing_led_count_bytes_overflow() {
        use crate::listener::parse_led_count_bytes;

        let bytes = [8, 32];
        let parsed_value = parse_led_count_bytes(bytes);
        assert_eq!(parsed_value, 2080)
    }
}
