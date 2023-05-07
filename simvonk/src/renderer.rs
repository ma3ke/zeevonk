use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use common::data::Data;

use crate::FRAMES_PER_SECOND;

pub(crate) fn render(receiver: Receiver<Data>) {
    let frame_time = Duration::from_secs_f64(1.0 / FRAMES_PER_SECOND);
    let mut data = receiver.recv().expect("channel recv error");
    let mut prev_time = Instant::now();
    loop {
        let elapsed = prev_time.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
        prev_time = Instant::now();

        for led in data.leds.iter() {
            let rgb = format!(
                "\u{1b}[48;2;{};{};{}m \u{1b}[0m",
                led.red, led.green, led.blue
            );
            eprint!("{rgb}");
        }
        eprintln!();

        // If there is new data available we use it for the next loop. Else we keep showing the
        // same pattern.
        if let Ok(new_data) = receiver.recv() {
            data = new_data;
        }
    }
}
