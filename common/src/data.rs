/// A Data type which holds the vector of bytes representing the values to send to the led strip.

///
/// Values are stored as an array of bytes. Each triplet of bytes holds the values to drive one
/// led, in (r, g, b) ordering.
#[derive(Clone, Debug)]
pub struct Data {
    leds: Vec<u8>,
}

impl Data {
    /// Number of leds stored inside data.
    ///
    /// Simply a thin wrapper for a call to len() of the internal vector storing the LED bytes,
    /// divided by 3. There are three bytes per LED.
    pub fn num_leds(&self) -> usize {
        self.leds.len() / 3
    }

    /// Creates a new Data struct when the data is well-formed.
    ///
    /// In case the number of bytes in data is not a multiple of 3, the data is not well formed and
    /// an Err is returned.
    pub fn from_bytes_vec(data: Vec<u8>) -> Result<Self, String> {
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
    pub fn led(&self, index: usize) -> (u8, u8, u8) {
        // TODO: There are more beautiful ways of doing this. Bikeshedding the playing around to
        // future self.
        if let [r, g, b] = self.leds[index * 3..index * 3 + 3] {
            (r, g, b)
        } else {
            panic!("malformed data: the number of leds must be a multiple of 3")
        }
    }
}
