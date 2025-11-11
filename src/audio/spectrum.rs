use rodio::Source;
use rustfft::{FftPlanner, num_complex::Complex};
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
    /// Creates a new spectrum capture wrapper around an audio source.
    ///
    /// # Arguments
    /// * `input` - The audio source to wrap
    /// * `buffer` - Shared buffer to store captured audio samples for FFT analysis
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
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
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

/// Updates spectrum analyzer bars by performing FFT on audio buffer.
///
/// Processes 4096 samples with Hann windowing into 64 logarithmic frequency bands.
/// Applies smoothing factors and gravity effect for smooth visualization.
/// # Arguments
/// * `spectrum_bars` - Mutable array of bar heights to update
/// * `fft_planner` - FFT planner for forward transform
/// * `audio_buffer` - Shared buffer containing recent audio samples
/// * `sample_rate` - Audio sample rate in Hz
/// * `channels` - Number of audio channels (mono/stereo)
#[allow(dead_code)]
pub fn update_spectrum_bars(
    spectrum_bars: &mut Vec<f32>,
    fft_planner: &mut FftPlanner<f32>,
    audio_buffer: &Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
    channels: u16,
) {
    const NUM_BARS: usize = 64;
    const FFT_SIZE: usize = 4096;

    let samples: Vec<f32> = {
        let buffer = audio_buffer.lock().unwrap();
        if buffer.len() < FFT_SIZE {
            return;
        }

        let mono_samples: Vec<f32> = if channels == 2 {
            buffer.iter()
                .copied()
                .collect::<Vec<f32>>()
                .chunks(2)
                .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
                .collect()
        } else {
            buffer.iter().copied().collect()
        };

        if mono_samples.len() < FFT_SIZE {
            return;
        }

        mono_samples[mono_samples.len() - FFT_SIZE..].to_vec()
    };

    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let window = 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / FFT_SIZE as f32).cos();
            Complex::new(sample * window, 0.0)
        })
        .collect();

    let fft = fft_planner.plan_fft_forward(FFT_SIZE);
    fft.process(&mut buffer);

    let nyquist = sample_rate as f32 / 2.0;
    let freq_per_bin = nyquist / (FFT_SIZE as f32 / 2.0);

    let freq_bands: Vec<(f32, f32)> = (0..NUM_BARS)
        .map(|i| {
            let freq_min = 20.0 * (20000.0_f32 / 20.0).powf(i as f32 / NUM_BARS as f32);
            let freq_max = 20.0 * (20000.0_f32 / 20.0).powf((i + 1) as f32 / NUM_BARS as f32);
            (freq_min, freq_max)
        })
        .collect();

    let mut last_bin_end = 0;

    for (i, &(freq_min, freq_max)) in freq_bands.iter().enumerate() {
        let mut bin_start = (freq_min / freq_per_bin) as usize;
        let mut bin_end = ((freq_max / freq_per_bin) as usize).min(FFT_SIZE / 2);

        if bin_start < last_bin_end {
            bin_start = last_bin_end;
        }

        if bin_end <= bin_start {
            bin_end = bin_start + 1;
        }

        last_bin_end = bin_end;

        let normalized = if bin_start >= FFT_SIZE / 2 || bin_end > FFT_SIZE / 2 {
            0.0
        } else {
            let bin_count = bin_end - bin_start;
            let mut magnitude_sum = 0.0_f32;
            for j in bin_start..bin_end {
                if j < buffer.len() {
                    let magnitude = (buffer[j].re * buffer[j].re + buffer[j].im * buffer[j].im).sqrt();
                    magnitude_sum += magnitude;
                }
            }

            if magnitude_sum.is_finite() && magnitude_sum > 0.0 {
                let avg_magnitude = magnitude_sum / bin_count as f32;
                let db = 20.0 * (avg_magnitude + 1e-10).log10();
                ((db + 110.0) / 160.0).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };

        let smoothing_up = 0.6;
        let smoothing_down = 0.85;

        let new_value = if normalized > spectrum_bars[i] {
            spectrum_bars[i] * (1.0 - smoothing_up) + normalized * smoothing_up
        } else {
            spectrum_bars[i] * (1.0 - smoothing_down) + normalized * smoothing_down
        };

        let gravity = if new_value < 0.05 {
            0.01
        } else {
            0.005
        };

        spectrum_bars[i] = (new_value - gravity).max(0.0);

        if spectrum_bars[i] < 0.001 {
            spectrum_bars[i] = 0.0;
        }
    }
}
