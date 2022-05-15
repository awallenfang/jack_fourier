use std::sync::{Mutex, Arc};
use std::thread;

use ringbuf::{Consumer};
use rustfft::num_traits::Pow;
use rustfft::{FftPlanner};
use rustfft::num_complex::Complex;
use crate::{BUFFER_SIZE, FFT_SIZE};

pub fn process_thread(mut consumer: Consumer<f32>, mut delivery_mutex: Arc<Mutex<Vec<f32>>>) {
    thread::spawn(move || {
            loop{
                // Loop until the ringbuffer has enough samples
                if consumer.len() >= BUFFER_SIZE {
                    // TODO: Carry over FFT like wolf
                    // NOTE: We always know the max length, so an array would be possible
                    // Init the buffer
                    let mut buffer = Vec::<Complex<f32>>::new();
                    
                    // And populate it with the samples from the ringbuffer
                    consumer.pop_each(|e| {
                        buffer.push(Complex{re:e  / (BUFFER_SIZE as f32 / 2.), im:0.0});
                        true
                    }, Some(BUFFER_SIZE));
                    
                    // Calculate the hann window and multiply it by the signal
                    for i in 0..BUFFER_SIZE {
                        buffer[i] *= hann_window(i, BUFFER_SIZE);
                    }

                    // Calculate how much 0s have to be padded and do so
                    let padding = FFT_SIZE - BUFFER_SIZE;

                    for _ in 0..padding {
                        buffer.push(Complex{re: 0.0, im: 0.0});
                    }

                    // fft set up
                    let mut planner = FftPlanner::<f32>::new();

                    let fft = planner.plan_fft_forward(FFT_SIZE);

                    fft.process(&mut buffer);

                    //TODO: Weird dropoff around 20k Hz. Maybe platform specific?
                    // Convert the complex signal into magnitudes
                    // Source: http://www.dspguide.com/ch8/8.htm
                    // NOTE: dB scaling source: https://github.com/wolf-plugins/wolf-spectrum/blob/master/src/Widgets/src/Spectrogram.cpp#L167
                    let magnitudes: Vec<f32> = buffer[0..(buffer.len() as f32 / 2.).floor() as usize + 1].iter()
                    .map(|e:&Complex<f32>| {
                        let real:f32 = e.re;
                        let imag:f32 = e.im;
                        (real.pow(2_i8) + imag.pow(2_i8)) * (2. / BUFFER_SIZE as f32)
                    })
                    .map(|e| {
                        10. * (e+1e-9).log10()
                    })
                    .map(|e| {
                        if e < -90. {
                            return -90.
                        }
                        e
                    })
                    .map(|e| {
                        1. - (e / -90.)
                    })
                    .map(|e| {
                        if e > 1. {
                            return 1.
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
        }
    );
}

/// A hann window for the size n
fn hann_window(i:usize, n: usize) -> f32 {
    0.5 * (1. - ((2. * std::f32::consts::PI * i as f32) / ((n-1) as f32)).cos())
}