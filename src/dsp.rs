use std::sync::{Arc, Mutex};
use std::thread;

use crate::{BUFFER_SIZE, FFT_SIZE};
use ringbuf::Consumer;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Pow;
use rustfft::FftPlanner;

pub fn process_thread(mut consumer: Consumer<f32>, delivery_mutex: Arc<Mutex<Vec<f32>>>) {
    thread::spawn(move || {
        loop {
            // Loop until the ringbuffer has enough samples
            if consumer.len() >= BUFFER_SIZE {
                // TODO: Carry over FFT like wolf
                // TODO: We always know the max length, so an array would be possible
                // TODO: Constant-Q transform
                // https://en.wikipedia.org/wiki/Constant-Q_transform
                // TODO: Research for other transforms with more exact low frequencies
                // Increasing the sample content adds more low frequency data

                // Init the buffer
                let mut buffer = Vec::<Complex<f32>>::new();

                // And populate it with the samples from the ringbuffer
                consumer.pop_each(
                    |e| {
                        buffer.push(Complex {
                            re: e / (BUFFER_SIZE as f32 / 2.),
                            im: 0.0,
                        });
                        true
                    },
                    Some(BUFFER_SIZE),
                );

                // Calculate the hann window and multiply it by the signal
                for (i, val) in buffer.iter_mut().enumerate().take(BUFFER_SIZE) {
                    *val *= hann_window(i, BUFFER_SIZE);
                }

                // Calculate how much 0s have to be padded and do so
                let padding = FFT_SIZE - BUFFER_SIZE;

                for _ in 0..padding {
                    buffer.push(Complex { re: 0.0, im: 0.0 });
                }

                // fft set up
                let mut planner = FftPlanner::<f32>::new();

                let fft = planner.plan_fft_forward(FFT_SIZE);

                fft.process(&mut buffer);

                // Convert the complex signal into magnitudes
                // Source: http://www.dspguide.com/ch8/8.htm
                let magnitudes: Vec<f32> = buffer
                    [0..(buffer.len() as f32 / 2.).floor() as usize + 1]
                    .iter()
                    .map(|e: &Complex<f32>| {
                        let real: f32 = e.re;
                        let imag: f32 = e.im;
                        real.pow(2_i8) + imag.pow(2_i8)
                    })
                    .map(|e| 10. * (e + 1e-9).log10())
                    .map(|e| {
                        if e < -90. {
                            return -90.;
                        }
                        e
                    })
                    .collect();

                // Send it to the UI through a mutex
                if let Ok(mut del) = delivery_mutex.lock() {
                    *del = magnitudes;
                }
            }
        }
    });
}

/// A hann window for the size n
fn hann_window(i: usize, n: usize) -> f32 {
    0.5 * (1. - ((2. * std::f32::consts::PI * i as f32) / ((n - 1) as f32)).cos())
}
