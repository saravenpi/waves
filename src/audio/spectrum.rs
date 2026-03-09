use rodio::Source;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct SpectrumCapture<I> {
    input: I,
    sample_rate: u32,
    channels: u16,
    buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl<I> SpectrumCapture<I>
where
    I: Source<Item = f32>,
{
    pub fn new(input: I, buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        let sample_rate = input.sample_rate();
        let channels = input.channels();
        Self {
            input,
            sample_rate,
            channels,
            buffer,
        }
    }
}

impl<I> Iterator for SpectrumCapture<I>
where
    I: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if let Some(sample) = self.input.next() {
            if let Ok(mut buffer) = self.buffer.lock() {
                buffer.push_back(sample);
                if buffer.len() > 8192 {
                    buffer.pop_front();
                }
            }
            Some(sample)
        } else {
            None
        }
    }
}

impl<I> Source for SpectrumCapture<I>
where
    I: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.input.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
