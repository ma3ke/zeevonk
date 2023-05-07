use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use common::listener::ChannelData;

use crate::FRAMES_PER_SECOND;

pub(crate) fn controller(receiver: Receiver<ChannelData>) {
    let frame_time = Duration::from_secs_f64(1.0 / FRAMES_PER_SECOND);
    let (_, mut data) = receiver.recv().expect("channel recv error");
    let mut prev_time = Instant::now();
    loop {
        let elapsed = prev_time.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
        prev_time = Instant::now();

        eprintln!("{:?}", data);

        // If there is new data available we use it for the next loop. Else we keep showing the
        // same pattern.
        if let Ok((_, new_data)) = receiver.recv() {
            data = new_data;
        }
    }
}
