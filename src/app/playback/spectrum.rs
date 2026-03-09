use crate::WavesApp;
use rustfft::num_complex::Complex;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

impl WavesApp {
    pub fn update_spectrum(&mut self, audio_buffer: &Arc<Mutex<VecDeque<f32>>>, sample_rate: u32, channels: u16) {
        const FFT_SIZE: usize = 4096;

        let samples: Vec<f32> = {
            let buffer = audio_buffer.lock().unwrap();
            if buffer.len() < FFT_SIZE {
                return;
            }

            let buffer_vec: Vec<f32> = buffer.iter().copied().collect();
            drop(buffer);

            let mono_samples: Vec<f32> = if channels == 2 {
                let required_samples = buffer_vec.len() / 2;
                if required_samples < FFT_SIZE {
                    return;
                }
                buffer_vec.chunks(2)
                    .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
                    .collect()
            } else {
                buffer_vec
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
                Complex::new(sample * self.hann_window[i], 0.0)
            })
            .collect();

        let fft = self.fft_planner.plan_fft_forward(FFT_SIZE);
        fft.process(&mut buffer);

        let nyquist = sample_rate as f32 / 2.0;
        let freq_per_bin = nyquist / (FFT_SIZE as f32 / 2.0);

        let mut last_bin_end = 0;

        for (i, &(freq_min, freq_max)) in self.freq_bands.iter().enumerate() {
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

            let new_value = if normalized > self.spectrum_bars[i] {
                self.spectrum_bars[i] * (1.0 - smoothing_up) + normalized * smoothing_up
            } else {
                self.spectrum_bars[i] * (1.0 - smoothing_down) + normalized * smoothing_down
            };

            let gravity = if new_value < 0.05 {
                0.01
            } else {
                0.005
            };

            self.spectrum_bars[i] = (new_value - gravity).max(0.0);

            if self.spectrum_bars[i] < 0.001 {
                self.spectrum_bars[i] = 0.0;
            }
        }
    }
}
