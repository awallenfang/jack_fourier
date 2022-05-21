use vizia::prelude::*;
use vizia::vg::{Paint, Path};

use crate::ui::bin::Bin;

pub struct Spectrometer {
    data: Vec<Bin>,
    sr: usize,
    style: Style,
    scale: Scale,
    col: vizia::vg::Color,
    min_freq: f32,
    max_freq: f32
}

pub enum VisEvents {
    Update(Vec<f32>),
    UpdateAttack(f32),
    UpdateRelease(f32),
    UpdateMin(f32),
    UpdateMax(f32)
}

#[allow(dead_code)]
pub enum Style {
    Spectrum,
    Gradient,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Scale {
    Linear,
    Root(f32),
    Logarithmic,
}

impl Spectrometer {
    pub fn new<L: Lens<Target = Vec<f32>>>(
        cx: &mut Context,
        lens: L,
        sampling_rate: usize,
        style: Style,
        scale: Scale,
        col: vizia::vg::Color,
    ) -> Handle<Self> {
        // Build the data vector and precompute all frequencies
        let mut data = vec![Bin::new(-90.); crate::FFT_SIZE];

        for (i, bin) in data.iter_mut().enumerate() {
            bin.set_frequency(bin2freq(i, crate::FFT_SIZE, sampling_rate));
        }

        

        Self {
            data,
            sr: sampling_rate,
            style,
            scale,
            col,
            min_freq: 20.,
            max_freq: sampling_rate as f32 / 2.
        }
        .build(cx, move |cx| {
            // Bind the input lens to the meter event to update the position
            Binding::new(cx, lens, |cx, value| {
                cx.emit(VisEvents::Update(value.get(cx)));
            });
        })
    }

    fn scale(&self, pos: f32) -> f32 {
        // NOTE: Maybe we can define a function that interpolates between a linear and a log scale
        match self.scale {
            Scale::Root(n) => map(
                pos.powf(n),
                self.min_freq.powf(n),
                self.max_freq.powf(n),
                0.,
                1.,
            ),
            Scale::Logarithmic => map(
                pos.log2(),
                self.min_freq.log2(),
                self.max_freq.log2(),
                0.,
                1.,
            ),
            Scale::Linear => map(pos, self.min_freq, self.max_freq, 0., 1.),
        }
    }
}

impl View for Spectrometer {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|e, _| match e {
            VisEvents::Update(data) => {
                for (i, val) in data.iter().enumerate() {
                    self.data[i].update(*val);
                }

                cx.style().needs_redraw = true;
            }
            VisEvents::UpdateAttack(x) => {
                self.data.iter_mut().for_each(|bin| bin.set_attack(*x));
            }
            VisEvents::UpdateRelease(x) => {
                self.data.iter_mut().for_each(|bin| bin.set_release(*x));
            }
            VisEvents::UpdateMin(x) => {
                self.min_freq = 20. + x * (self.sr as f32 / 4.);
            },
            VisEvents::UpdateMax(x) => {
                self.max_freq = (self.sr as f32 / 2.) - (1. - x) * (self.sr as f32 / 2.);
            },
        });
    }

    fn draw(&self, cx: &mut DrawContext<'_>, canvas: &mut Canvas) {
        let entity = cx.current();

        let bounds = cx.cache().get_bounds(entity);

        //Skip meters with no width or no height
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let width = bounds.w;
        let height = bounds.h;

        let data = self.data.clone();

        // Still not working T.T
        // I give up for now

        // This is old code where data was just a vec of f32s
        // for (i,val) in data.iter_mut().enumerate() {
        //     let mut new_val = *val;
        //     if new_val > -89. {
        //         let octave = bin2freq(i, self.data.len(), self.sr).log2();
        //         new_val += (octave) * self.slope;

        //         if new_val > 0. {

        //             new_val = 0.;
        //         }
        //         *val = new_val;
        //     }
        // }

        //TODO: 4.5dB dropoff pink noise
        //https://www.reddit.com/r/audioengineering/comments/agcr8d/i_ran_whitepink_noise_through_my_system_and/

        match self.style {
            Style::Spectrum => {
                let mut line_path = Path::new();
                
                let mut first_bin_reached = false;
                let mut last_bin_reached = false;
                let mut bin_before_first_bin = self.data[0];
                
                for bin in data {
                    // TODO: sinc interpolation
                    // Logarithmic scaling
                    // Source: https://mu.krj.st/spectrm/

                    // If the first bin hasn't been reached yet
                    if !first_bin_reached {
                        // Check if the new bin is in the region. If it is, then the saved bin is the one just outside of the window
                        if bin.get_frequency() > self.min_freq && bin.get_frequency() < self.max_freq {
                            // Set the start to the one outside the window
                            // TODO: Interpolate this for the correct value
                            line_path.move_to(0., map(bin_before_first_bin.get_smooth_val(), 0., -90., 0., 1.) * height);

                            let position = self.scale(bin.get_frequency()) * width;
                            let y_pos = map(bin.get_smooth_val(), 0., -90., 0., 1.);
                            line_path.line_to(position, y_pos * height);

                            first_bin_reached = true;
                        } else {
                            bin_before_first_bin = bin.clone();
                        }
                        continue
                    }
                    if !last_bin_reached {
                        if bin.get_frequency() < self.max_freq {
                            // Set the start to the one outside the window
                            // TODO: Interpolate this for the correct value
                            let position = self.scale(bin.get_frequency()) * width;
                            let y_pos = map(bin.get_smooth_val(), 0., -90., 0., 1.);
                            line_path.line_to(position, y_pos * height);
                        } else {
                            line_path.move_to(width, map(bin.get_smooth_val(), 0., -90., 0., 1.) * height);

                            last_bin_reached = true;
                        }
                    }
                    // if bin.get_frequency() > self.min_freq && bin.get_frequency() < self.max_freq{
                    //     let position = self.scale(bin.get_frequency()) * width;
                    //     let y_pos = map(bin.get_smooth_val(), 0., -90., 0., 1.);
                    //     line_path.line_to(position, y_pos * height);
                    // }
                }

                let mut line_paint = Paint::color(self.col);
                // let mut line_paint = Paint::color(Color::hex("#f54e47"));
                line_paint.set_line_width(2.0);

                canvas.stroke_path(&mut line_path, line_paint);
            }
            Style::Gradient => {
                //TODO: Gradient
                let mut color_vec: Vec<(f32, vizia::vg::Color)> = Vec::new();
                // Split into 16px wide rectangles that are seperately gradiented
                // Util function to go [0,1] to bin, since the bins are overfitting

                for bin in data {
                    let position = self.scale(bin.get_frequency()) * width;

                    color_vec.push((position, gradient_color_map(bin.get_smooth_val())));
                }

                let paint = Paint::linear_gradient_stops(0.0, 0.0, width, 0.0, &color_vec);

                let mut path = Path::new();
                path.rect(0.0, 0.0, width, height);

                canvas.fill_path(&mut path, paint);
            }
        }
    }
}


pub trait SpectrometerHandle {
    fn attack(self, val: impl Res<f32>) -> Self;
    fn release(self, val: impl Res<f32>) -> Self;
    fn min(self, val: impl Res<f32>) -> Self;
    fn max(self, val: impl Res<f32>) -> Self;
}

impl SpectrometerHandle for Handle<'_, Spectrometer> {
    fn attack(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            cx.emit_to(entity, VisEvents::UpdateAttack(value));
        });

        self
    }

    fn release(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            cx.emit_to(entity, VisEvents::UpdateRelease(value));
        });

        self
    }

    fn min(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            cx.emit_to(entity, VisEvents::UpdateMin(value));
        });

        self
    }

    fn max(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            cx.emit_to(entity, VisEvents::UpdateMax(value));
        });

        self
    }
}



/// Converts the bin index to a frequency in Hz
///
/// Source: https://mu.krj.st/spectrm/
fn bin2freq(bin_idx: usize, bin_amt: usize, sample_rate: usize) -> f32 {
    bin_idx as f32 * (sample_rate as f32 / (2. * bin_amt as f32))
}

/// Maps [x0,x1] to [y0,y1] linearly at position val in [x0,x1]
///
/// Source: https://tig.krj.st/spectrm/file/spectrm.c
/// Line 102
fn map(val: f32, x0: f32, x1: f32, y0: f32, y1: f32) -> f32 {
    y0 + (y1 - y0) * (val - x0) / (x1 - x0)
}

fn gradient_color_map(val: f32) -> vizia::vg::Color {
    let col = vizia::vg::Color::rgb((val * 255.) as u8, (val * 255.) as u8, (val * 255.) as u8);
    if col.r != 0.0 {}
    col
    // vizia::vg::Color::white()
}
