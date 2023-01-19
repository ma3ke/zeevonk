use std::sync::mpsc;
use std::thread;

use crate::controller::controller;
use crate::data::Data;
use crate::listener::listener;
use crate::stats::Stats;

mod controller;
mod data;
mod listener;
mod stats;

const ADDRESS: &str = "0.0.0.0:80";
const GPIO_PIN: i32 = 10; // GPIO 10 = SPI0 MOSI
const FRAMES_PER_SECOND: f64 = 50.0;
const WELCOME_MESSAGE: &str = r#"
                                                       888
    8P d8P  ,e e,   ,e e,  Y8b Y888P  e88 88e  888 8e  888 ee
    P d8P  d88 88b d88 88b  Y8b Y8P  d888 888b 888 88b 888 P
     d8P d 888   , 888   ,   Y8b "   Y888 888P 888 888 888 b
    d8P d8  "KoeN"  "BauX"    Y8P     "88 88"  888 888 888 8b
    
                By Koen & Bauke Westendorp, 2023.
"#;

/// A ConnectionInformation type which is sent through a channel to communicate the number of open
/// connections and the sender's client_id to the receiving led driver thread. These values are
/// used for logging.
#[derive(Clone, Copy, Debug)]
struct ConnectionInformation {
    client_id: u32,
    open_connections: u32,
}

impl ConnectionInformation {
    /// Creates a new Connection struct.
    fn new(client_id: u32, open_connections: u32) -> Self {
        Self {
            client_id,
            open_connections,
        }
    }
}

/// A tuple to be sent over the channel, containing the ConnectionInformation metadata and the Data
/// itself.
type ChannelData = (ConnectionInformation, Data);

fn main() {
    println!("{WELCOME_MESSAGE}");
    let (sender, receiver) = mpsc::channel::<(ConnectionInformation, Data)>();

    thread::spawn(move || controller(receiver));
    listener(ADDRESS, sender)
}
