use std::net::TcpListener;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::mpsc::Sender;
use std::thread;

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message,
};

use crate::{ChannelData, ConnectionInformation, Data};

/// Listens for new websocket connections on `address` and spawns a new thread to listen on for
/// each connection. When a connection receives data, it is sent over the `sender` channel handler.
pub(crate) fn listener(address: &str, sender: Sender<ChannelData>) {
    static CLIENT_ID: AtomicU32 = AtomicU32::new(0);
    static OPEN_CONNECTIONS: AtomicU32 = AtomicU32::new(0);

    let server = TcpListener::bind(address).unwrap();
    println!("Listening on: {address} ...\n");
    for stream in server.incoming() {
        let sender = sender.clone();
        let client_id = CLIENT_ID.fetch_add(1, Relaxed);
        // let open_connections = OPEN_CONNECTIONS.clone();
        thread::spawn(move || {
            let callback = |_req: &Request, response: Response| {
                let open_connections = OPEN_CONNECTIONS.load(Relaxed);
                println!("({open_connections}) client {client_id:>2}: New connection.");
                Ok(response)
            };

            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();
            // Now that we have opened the connection, we add one to the open connectsions counter.
            OPEN_CONNECTIONS.fetch_add(1, Relaxed);

            'receive: loop {
                match websocket.read_message().unwrap() {
                    Message::Binary(bytes) => {
                        let data = Data::from_bytes_vec(bytes.to_vec()).unwrap();
                        print!(".");
                        let connection =
                            ConnectionInformation::new(client_id, OPEN_CONNECTIONS.load(Relaxed));
                        sender.send((connection, data)).expect("channel send error");
                    }
                    Message::Text(t) => println!("client {client_id:>2}: text: {t}"),
                    Message::Ping(_) => println!("client {client_id:>2}: ping"),
                    Message::Pong(_) => println!("client {client_id:>2}: pong"),
                    Message::Close(_) => {
                        let open_connections = OPEN_CONNECTIONS.load(Relaxed);
                        let new_connections = open_connections - 1;
                        println!("({open_connections}) {client_id:>2}: connection closed ({open_connections} -> {new_connections})");
                        break 'receive;
                    }
                    Message::Frame(_) => println!("client {client_id:>2}: frame"),
                };
            }

            // When the connection closes, we decrement OPEN_CONNECTIONS by one.
            OPEN_CONNECTIONS.fetch_sub(1, Relaxed);
        });
    }
}
