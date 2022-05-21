use vizia::prelude::*;
use vizia::vg::{Paint, Path};

const C_FREQUENCIES: &[f32] = &[
    16.352, 32.703, 65.406, 130.813, 261.626, 523.251, 1046.502, 2093.005, 4186.009, 8372.018,
    16744.036,
];

enum FreqEvents {
    UpdateMin(f32),
    UpdateMax(f32),
}

#[allow(dead_code)]
pub struct FrequencyMarkers {
    min_freq: f32,
    max_freq: f32,
    sr: f32,
}

impl FrequencyMarkers {
    pub fn new(cx: &mut Context, sampling_rate: usize) -> Handle<Self> {
        Self {
            min_freq: 20.,
            max_freq: sampling_rate as f32 / 2.,
            sr: sampling_rate as f32,
        }
        .build(cx, |_cx| {})
    }

    /// Maps [x0,x1] to [y0,y1] linearly at position val in [x0,x1]
    ///
    /// Source: https://tig.krj.st/spectrm/file/spectrm.c
    /// Line 102
    fn map(&self, val: f32, x0: f32, x1: f32, y0: f32, y1: f32) -> f32 {
        y0 + (y1 - y0) * (val - x0) / (x1 - x0)
    }

    fn freq_to_pos(&self, freq: f32) -> f32 {
        self.map(
            freq.log2(),
            self.min_freq.log2(),
            self.max_freq.log2(),
            0.,
            1.,
        )
    }
}

impl View for FrequencyMarkers {
    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        event.map(|e, _| match e {
            FreqEvents::UpdateMin(x) => {
                self.min_freq = 20. + x * (self.sr as f32 / 4.);
            }
            FreqEvents::UpdateMax(x) => {
                self.max_freq = (self.sr as f32 / 2.) - (1. - x) * (self.sr as f32 / 2.);
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

        let line_paint = Paint::color(vizia::vg::Color::hex("#565454"));

        let text_paint = Paint::color(vizia::vg::Color::white());

        let mut path = Path::new();

        for (c_idx, freq) in C_FREQUENCIES.iter().enumerate() {
            if *freq < self.min_freq || *freq > self.max_freq {
                continue;
            }

            let freq_text = format!("C{}", c_idx);

            let x_pos = self.freq_to_pos(*freq) * width;

            path.move_to(x_pos, 0.);
            path.line_to(x_pos, height);
            let res = canvas.fill_text(x_pos, height, &freq_text, text_paint);

            match res {
                Ok(_) => {}
                Err(_) => {
                    println!("Failed to write frequency labels.")
                }
            };
        }

        canvas.stroke_path(&mut path, line_paint);
    }
}

pub trait FreqMarkerHandle {
    fn min(self, val: impl Res<f32>) -> Self;
    fn max(self, val: impl Res<f32>) -> Self;
}

impl FreqMarkerHandle for Handle<'_, FrequencyMarkers> {
    fn min(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            cx.emit_to(entity, FreqEvents::UpdateMin(value));
        });

        self
    }

    fn max(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            cx.emit_to(entity, FreqEvents::UpdateMax(value));
        });

        self
    }
}
