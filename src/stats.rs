/// A Stats type which can track the minimum and maximum value encountered and the average value
/// over the past `N` encountered values.
///
/// Internally, the `buffer` is basically a ring buffer.
pub(crate) struct Stats<T, const N: usize> {
    min: T,
    max: T,
    buffer: [T; N],
    cursor: usize,
}

impl<T, const N: usize> Stats<T, N>
where
    T: Copy + Default + PartialOrd,
{
    /// Create new Stats.
    pub(crate) fn new() -> Self {
        Self {
            min: T::default(),
            max: T::default(),
            buffer: [T::default(); N],
            cursor: 0,
        }
    }

    /// Push a value to the buffer.
    ///
    /// When the buffer is full, the oldest value is overridden.
    pub(crate) fn push(&mut self, value: T) {
        self.buffer[self.cursor] = value;
        self.cursor = (self.cursor + 1) % N;
        if self.min > value {
            self.min = value
        }
        if self.max > value {
            self.max = value
        }
    }

    /// Return the minimum encountered value.
    pub(crate) fn min(&self) -> T {
        self.min
    }

    /// Return the maximum encountered value.
    pub(crate) fn max(&self) -> T {
        self.max
    }

    pub(crate) fn buffer(&self) -> [T; N] {
        self.buffer
    }

    pub(crate) fn buffer_size(&self) -> usize {
        N
    }
}
