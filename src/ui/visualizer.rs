use vizia::prelude::*;
use vizia::vg::{Color, Paint, Path, ImageFlags};
use image;
pub struct Spectrometer {
    data: Vec<f32>,
    sr: usize,
    style: Style,
    scale: Scale,
    col: vizia::vg::Color,
    smoothing_factor: f32
}

pub enum VisEvents {
    Update(Vec<f32>)
}

pub enum Style {
    Spectrum,
    Gradient
}

#[derive(Clone, Copy)]
pub enum Scale {
    Linear,
    Root(f32),
    Logarithmic,
}

impl Spectrometer {
    pub fn new<L: Lens<Target = Vec<f32>>>(cx: &mut Context, lens: L, sampling_rate: usize, style: Style, scale:Scale, col: vizia::vg::Color, smoothing_factor: f32) -> Handle<Self> {
        Self {
            data: lens.get(cx),
            sr: sampling_rate,
            style,
            scale,
            col,
            smoothing_factor
        }
        .build(cx, move |cx| {
            // Bind the input lens to the meter event to update the position
            Binding::new(cx, lens, |cx, value| {
                cx.emit(VisEvents::Update(value.get(cx)));
            });
        })
    }
}

impl View for Spectrometer {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|e, _| {
            match e {
                VisEvents::Update(data) => {
                    let new_data = data.clone();

                    for i in 0..(new_data.len()) {
                        self.data[i] -= self.smoothing_factor * (self.data[i] - new_data[i]);
                    }

                    cx.style().needs_redraw = true;
                }
            }
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

        let data = &self.data;

        //TODO: 4.5dB dropoff pink noise
        
        match self.style {
            Style::Spectrum => {
                let mut line_path = Path::new();
                line_path.move_to(0.0, height);

                let mut position = 0. as f32;

                for i in 1..data.len() {
                    // TODO: sinc interpolation
                    // Logarithmic scaling
                    // Source: https://mu.krj.st/spectrm/
                    position = scale(bin2freq(i, data.len(), self.sr), self.scale, self.sr, width);

                    line_path.line_to(position, (1. - data[i]) * height);
                }

                
                let mut line_paint = Paint::color(self.col);
                // let mut line_paint = Paint::color(Color::hex("#f54e47"));
                line_paint.set_line_width(3.0);

                canvas.stroke_path(&mut line_path, line_paint);
            }
            Style::Gradient => {
                //TODO: Gradient
                let mut color_vec: Vec<(f32, vizia::vg::Color)> = Vec::new();
                // Split into 16px wide rectangles that are seperately gradiented
                // Util function to go [0,1] to bin, since the bins are overfitting

                for i in 1..data.len() {
                     let position = map(
                         lin2log(
                             bin2freq(i, data.len(), self.sr), 
                                 20., self.sr as f32 / 2.), 
                         0., 1., 0., width);

                    color_vec.push((position, gradient_color_map(data[i])));
                }

                let paint = Paint::linear_gradient_stops(0.0, 0.0, width, 0.0, &color_vec);

                let mut path = Path::new();
                path.rect(0.0, 0.0, width, height);

                canvas.fill_path(&mut path, paint);

            }
        }
    }
}

fn scale(pos: f32, scale_type: Scale, sr: usize, width: f32) -> f32 {
    // NOTE: Maybe we can define a function that interpolates between a linear and a log scale
    match scale_type {
        Scale::Root(n) => {
            map(pos.powf(n), 20.0_f32.powf(n), (sr as f32 / 2.).powf(n), 0., width)
        }
        Scale::Logarithmic => {
            map(pos.log10(), 20.0_f32.log10(), (sr as f32 / 2.).log10(),  0., width)
        }
        Scale::Linear => {
            pos
        }
    }
}

/// Converts the bin index to a frequency position in [0,1], where 1 is half the sampling rate according to the Nyquist Theorem
/// 
/// Source: https://mu.krj.st/spectrm/
fn bin2freq(bin_idx: usize, bin_amt: usize, sample_rate: usize) -> f32 {
    bin_idx as f32 * (sample_rate as f32 / bin_amt as f32)
}

/// Maps linear values logarithmically between min and max frequency values, where everything is a frequency in [0,1] times half the sampling rate
/// 
/// Source: https://mu.krj.st/spectrm/
fn lin2log(val: f32, min: f32, max:f32) -> f32 {
    map(val.log10(), min.log10(), max.log10(), 0., 1.)
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
    if col.r != 0.0 {
        println!("{:?}", col);

    }
    col    
    // vizia::vg::Color::white()
}