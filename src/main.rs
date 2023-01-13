#![feature(array_chunks)]
#![feature(slice_as_chunks)]

use std::net::TcpListener;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc;
use std::thread::spawn;

use rs_ws281x::{ChannelBuilder, ControllerBuilder, StripType};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message,
};

const ADDRESS: &str = "0.0.0.0:80";

#[derive(Clone, Debug)]
struct Data {
    #[allow(unused)]
    num_frames: u8,
    num_leds: u8,
    framerate: u8,
    leds: Vec<u8>,
}

impl Data {
    fn from_bytes_vec(data: Vec<u8>) -> Result<Self, String> {
        if data.len() <= 3 {
            return Err("data must contain a 3 byte header (n_frames, n_leds, framerate) followed by bytes describing the led states".to_string());
        }

        let mut data = data.iter();
        let num_frames = *data
            .next()
            .ok_or("invalid field 'num_frames'".to_string())?;
        let num_leds = *data.next().ok_or("invalid field 'num_leds'".to_string())?;
        let framerate = *data.next().ok_or("invalid field 'framerate'".to_string())?;
        let leds: Vec<u8> = data.map(|b| *b).collect();

        if leds.len() != num_frames as usize * num_leds as usize * 3 {
            return Err(format!(
                "frames data must conform to the size specified in the header ({} != {})",
                leds.len(),
                num_frames as usize * num_leds as usize * 3
            )
            .to_string());
        };

        Ok(Self {
            num_frames,
            num_leds,
            framerate,
            leds,
        })
    }

    fn frames(&self) -> Vec<&[[u8; 3]]> {
        self.leds
            .chunks(self.num_leds as usize * 3)
            .map(|frame| {
                let (leds, _) = frame.as_chunks::<3>();
                leds
            })
            .collect::<Vec<&[[u8; 3]]>>()
    }
}

fn main() {
    let (sender, receiver) = mpsc::channel::<(u32, Data)>();

    let _controller_handle = spawn(move || {
        let mut controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0, // Channel Index
                ChannelBuilder::new()
                    .pin(10) // GPIO 10 = SPI0 MOSI
                    .count(208) // Number of LEDs
                    .invert(false)
                    .strip_type(StripType::Ws2811Gbr)
                    .brightness(255) // default: 255
                    .build(),
            )
            .build()
            .unwrap();

        let (mut conn, mut data) = receiver.recv().expect("channel recv error");
        loop {
            let time_per_frame = std::time::Duration::from_secs_f64(1.0 / data.framerate as f64);
            let mut prev_time = std::time::Instant::now();

            for (_n, frame) in data.frames().iter().enumerate() {
                let leds_mut = controller.leds_mut(0);
                for i in 0..data.num_leds as usize {
                    let ctrl_led = &mut leds_mut[i];
                    let [c1, c2, c3] = frame[i];
                    *ctrl_led = [c1, c2, c3, 0];
                }
                controller.render().unwrap();

                let elapsed = prev_time.elapsed();
                print!("{conn:03}: {:.3} ms\r", elapsed.as_millis());
                if elapsed < time_per_frame {
                    std::thread::sleep(time_per_frame - elapsed);
                }
                prev_time = std::time::Instant::now();
            }

            // If there is new data available we use it for the next loop. Else we keep showing the
            // same pattern.
            if let Ok((new_conn, new_data)) = receiver.try_recv() {
                conn = new_conn;
                data = new_data;
            }
        }
    });

    let conn = AtomicU32::new(0);

    let server = TcpListener::bind(ADDRESS).unwrap();
    for stream in server.incoming() {
        let sender = sender.clone();
        let conn = conn.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        spawn(move || {
            let callback = |req: &Request, response: Response| {
                println!("{conn:03}: Received a new ws handshake");
                println!("{conn:03}: The request's path is: {}", req.uri().path());
                println!("{conn:03}: The request's headers are:");
                for (ref header, _value) in req.headers() {
                    println!("{conn:03}: * {}", header);
                }

                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

            loop {
                let msg = websocket.read_message().unwrap();
                // websocket
                //     .write_message(Message::Text("received".to_string()))
                //     .unwrap();

                match msg {
                    Message::Binary(bytes) => {
                        let data = Data::from_bytes_vec(bytes.to_vec()).unwrap();
                        sender.send((conn, data)).expect("channel send error");
                    }
                    Message::Text(t) => println!("{conn:03}: text: {t}"),
                    Message::Ping(_) => println!("{conn:03}: ping"),
                    Message::Pong(_) => println!("{conn:03}: pong"),
                    Message::Close(_) => {
                        println!("{conn:03}: close");
                        break;
                    }
                    Message::Frame(_) => println!("{conn:03}: frame"),
                };
            }
        });
    }
}
