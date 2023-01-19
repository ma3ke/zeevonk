use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use rs_ws281x::{ChannelBuilder, ControllerBuilder, StripType, WS2811Error};

use crate::{ChannelData, Stats, FRAMES_PER_SECOND, GPIO_PIN};

/// Initializes and runs the led strip controller. When new data is received over the `receiver`
/// channel handler, it is rendered to the led strip controller on the tempo of FRAMES_PER_SECOND.
///
/// Returns [rs_ws281x::WS2811Error] in case it is returned by internal workings.
/// Otherwise, this function will not return.
///
/// When no new messages are sent over the channel, the last frame will be displayed continuously.
pub(crate) fn controller(receiver: Receiver<ChannelData>) -> Result<(), WS2811Error> {
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
    let (mut connection, mut data) = receiver.recv().expect("channel recv error");
    let mut prev_time = Instant::now();
    let mut stats: Stats<f64, 32> = Stats::new();
    loop {
        let leds_mut = controller.leds_mut(0);

        for i in 0..data.num_leds() as usize {
            let ctrl_led = &mut leds_mut[i];
            let (r, g, b) = data.led(i);
            *ctrl_led = [r, g, b, 0];
        }

        controller.render()?;

        let open_connections = connection.open_connections;
        let client_id = connection.client_id;
        let (r, g, b) = data.led(data.num_leds() - 1);
        let elapsed = prev_time.elapsed();
        // This escape code magic prints a nice colored cell to a color-enabled console.
        let rgb = format!("\u{1b}[48;2;{r};{g};{b}m \u{1b}[0m");
        let elapsed_millis = elapsed.as_micros() as f32 / 1000.0;
        stats.push(elapsed.as_millis() as f64);
        let avg = stats.buffer().iter().sum::<f64>() / stats.buffer_size() as f64;
        let min = stats.min();
        let max = stats.max();
        println!("({open_connections}) client {client_id:>2}: {elapsed_millis:.2} ms ({min:.2}<{avg:.2}<{max:.2}) [{rgb}]",);
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
