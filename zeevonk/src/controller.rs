use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use common::data::Data;

use rs_ws281x::{ChannelBuilder, ControllerBuilder, StripType, WS2811Error};

use crate::{FRAMES_PER_SECOND, GPIO_PIN};

/// Initializes and runs the led strip controller. When new data is received over the `receiver`
/// channel handler, it is rendered to the led strip controller on the tempo of FRAMES_PER_SECOND.
///
/// Returns [rs_ws281x::WS2811Error] in case it is returned by internal workings.
/// Otherwise, this function will return nothing.
///
/// When no new messages are sent over the channel, the last frame will be displayed continuously.
pub(crate) fn controller(receiver: Receiver<Data>) -> Result<(), WS2811Error> {
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
        .build()?;

    let frame_time = Duration::from_secs_f64(1.0 / FRAMES_PER_SECOND);
    let mut data = receiver.recv().expect("channel recv error");
    let mut prev_time = Instant::now();
    loop {
        let leds_mut = controller.leds_mut(0);

        for (i, led) in data.leds.iter().enumerate() {
            leds_mut[i] = [led.red, led.green, led.blue, 0];
        }

        controller.render()?;

        // This escape code magic prints a nice colored cell to a color-enabled console.
        let led = data.leds[data.leds.len() - 1];
        let rgb = format!(
            "\u{1b}[48;2;{};{};{}m \u{1b}[0m",
            led.red, led.green, led.blue
        );
        eprintln!("{rgb}");

        let elapsed = prev_time.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
        prev_time = Instant::now();

        // If there is new data available we use it for the next loop. Else we keep showing the
        // same pattern.
        if let Ok(new_data) = receiver.recv() {
            data = new_data;
        }
    }
}
