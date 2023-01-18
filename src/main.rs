#![feature(array_chunks)]
#![feature(slice_as_chunks)]

use std::net::TcpListener;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use rs_ws281x::{ChannelBuilder, ControllerBuilder, StripType};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message,
};

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

/// A Data type which holds the vector of bytes representing the values to send to the led strip.
///
/// Values are stored as an array of bytes. Each triplet of bytes holds the values to drive one
/// led, in (r, g, b) ordering.
#[derive(Clone, Debug)]
struct Data {
    leds: Vec<u8>,
}

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

impl Data {
    /// Number of leds stored inside data.
    ///
    /// Simply a thin wrapper for a call to len() of the internal vector storing the LED bytes,
    /// divided by 3. There are three bytes per LED.
    fn num_leds(&self) -> usize {
        self.leds.len() / 3
    }

    /// Creates a new Data struct when the data is well-formed.
    ///
    /// In case the number of bytes in data is not a multiple of 3, the data is not well formed and
    /// an Err is returned.
    fn from_bytes_vec(data: Vec<u8>) -> Result<Self, String> {
        match data.len() % 3 {
            0 => Ok(Self { leds: data }),
            _ => Err("data should have a length that is a multiple of three, considering there are 3 values for each led".to_string()),
        }
    }

    /// Returns a tuple of the (red, green, blue) values for the led at index.
    ///
    /// This method will panic if the data is malformed. Because we deal with the shape of the data
    /// in the initialization using `from_bytes_vec`, this should not happen. If it does, there is
    /// something fishy going on.
    fn led(&self, index: usize) -> (u8, u8, u8) {
        // TODO: There are more beautiful ways of doing this. Bikeshedding the playing around to
        // future self.
        if let [r, g, b] = self.leds[index * 3..index * 3 + 3] {
            (r, g, b)
        } else {
            panic!("malformed data: the number of leds must be a multiple of 3")
        }
    }
}

/// Initializes and runs the led strip controller. When new data is received over the `receiver`
/// channel handler, it is
fn controller(receiver: Receiver<ChannelData>) {
    let mut controller = ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0, // Channel Index
            ChannelBuilder::new()
                .pin(GPIO_PIN)
                .count(208) // Number of LEDs
                .invert(false)
                .strip_type(StripType::Ws2811Gbr)
                .brightness(255) // default: 255
                .build(),
        )
        .build()
        .unwrap();

    let frame_time = Duration::from_secs_f64(1.0 / FRAMES_PER_SECOND);
    let (mut connection, mut data) = receiver.recv().expect("channel recv error");
    let mut prev_time = Instant::now();
    loop {
        let leds_mut = controller.leds_mut(0);

        for i in 0..data.num_leds() as usize {
            let ctrl_led = &mut leds_mut[i];
            let (r, g, b) = data.led(i);
            *ctrl_led = [r, g, b, 0];
        }

        controller.render().unwrap();

        let open_connections = connection.open_connections;
        let client_id = connection.client_id;
        let (r, g, b) = data.led(data.num_leds() - 1);
        let elapsed = prev_time.elapsed();
        let rgb = format!("\u{1b}[48;2;{r};{g};{b}m \u{1b}[0m");
        let elapsed_millis = elapsed.as_micros() as f32 / 1000.0;
        println!("({open_connections}) client {client_id:>2}: {elapsed_millis:.2} ms  [{rgb}]",);
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
        prev_time = Instant::now();

        // If there is new data available we use it for the next loop. Else we keep showing the
        // same pattern.
        if let Ok((new_connection, new_data)) = receiver.recv() {
            connection = new_connection;
            data = new_data;
        }
    }
}

/// Listens for new websocket connections on `address` and spawns a new thread to listen on for
/// each connection. When a connection receives data, it is sent over the `sender` channel handler.
fn listener(address: &str, sender: Sender<ChannelData>) {
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
                let msg = websocket.read_message().unwrap();
                // websocket
                //     .write_message(Message::Text("received".to_string()))
                //     .unwrap();

                match msg {
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

fn main() {
    println!("{WELCOME_MESSAGE}");
    let (sender, receiver) = mpsc::channel::<(ConnectionInformation, Data)>();

    thread::spawn(move || controller(receiver));
    listener(ADDRESS, sender)
}
