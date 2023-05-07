#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// A Data type which holds the vector of bytes representing the values to send to the led strip.
///
/// Values are stored as an array of bytes. Each triplet of bytes holds the values to drive one
/// led, in (r, g, b) ordering.
#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    pub leds: Vec<Color>,
}

impl Data {
    /// Creates a new Data struct when the data is well-formed.
    ///
    /// In case the number of bytes in data is not a multiple of 3, the data is not well formed and
    /// an Err is returned.
    pub fn from_bytes_vec(bytes: Vec<u8>) -> Result<Self, String> {
        let mut bytes = bytes;
        match bytes.len() % 3 {
            0 => {
                let mut colors = Vec::<Color>::new();
                while bytes.len() > 2 {
                    let rgb: Vec<u8> = bytes.drain(0..3).collect();
                    if let [red, green, blue] = rgb.as_slice().to_owned()[..] {
                        colors.push(Color { red, green, blue });
                    } else {
                        eprintln!("Failed to get rgb values from bytes!");
                    }
                }

                Ok(Self { leds: colors})
            },
            _ => Err("data should have a length that is a multiple of three, considering there are 3 values for each led".to_string()),
        }
    }
}

mod tests {
    #[test]
    fn parsing_led_bytes() {
        use crate::data::{Color, Data};

        let bytes = vec![72, 39, 100, 95, 26, 200, 122, 102, 120];
        let parsed_colors = Data::from_bytes_vec(bytes).unwrap();

        assert_eq!(
            parsed_colors,
            Data {
                leds: vec![
                    Color {
                        red: 72,
                        green: 39,
                        blue: 100
                    },
                    Color {
                        red: 95,
                        green: 26,
                        blue: 200
                    },
                    Color {
                        red: 122,
                        green: 102,
                        blue: 120
                    },
                ]
            }
        )
    }
}
