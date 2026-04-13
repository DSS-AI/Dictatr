use std::collections::VecDeque;

/// Sliding-window audio buffer for 16 kHz mono f32 samples.
/// Drops oldest samples when capacity is exceeded.
pub struct RingBuffer {
    buf: VecDeque<f32>,
    capacity: usize,
}

impl RingBuffer {
    pub fn with_seconds(seconds: u32, sample_rate: u32) -> Self {
        Self::with_capacity((seconds as usize) * (sample_rate as usize))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for s in samples {
            if self.buf.len() == self.capacity {
                self.buf.pop_front();
            }
            self.buf.push_back(*s);
        }
    }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn is_full(&self) -> bool { self.buf.len() == self.capacity }

    pub fn drain_to_vec(&mut self) -> Vec<f32> {
        self.buf.drain(..).collect()
    }

    pub fn clear(&mut self) { self.buf.clear(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_from_seconds() {
        let rb = RingBuffer::with_seconds(2, 16_000);
        assert_eq!(rb.len(), 0);
    }

    #[test]
    fn drops_oldest_when_full() {
        let mut rb = RingBuffer::with_capacity(4);
        rb.push_samples(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(rb.len(), 4);
        let v = rb.drain_to_vec();
        assert_eq!(v, vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn drain_empties_buffer() {
        let mut rb = RingBuffer::with_capacity(8);
        rb.push_samples(&[1.0, 2.0]);
        let _ = rb.drain_to_vec();
        assert!(rb.is_empty());
    }

    #[test]
    fn is_full_reports_correctly() {
        let mut rb = RingBuffer::with_capacity(3);
        assert!(!rb.is_full());
        rb.push_samples(&[1.0, 2.0, 3.0]);
        assert!(rb.is_full());
    }
}
