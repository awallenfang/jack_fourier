use vizia::prelude::*;
use vizia::vg::{Paint, Path};

const C_FREQUENCIES: &[f32] = &[
    16.352, 32.703, 65.406, 130.813, 261.626, 523.251, 1046.502, 2093.005, 4186.009, 8372.018,
    16744.036,
];

pub struct FrequencyMarkers {
    min: f32,
    max: f32,
    sr: f32,
}

impl FrequencyMarkers {
    pub fn new(cx: &mut Context, sampling_rate: usize) -> Handle<Self> {
        Self {
            min: 20.,
            max: 20000.,
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
        self.map(freq.log2(), self.min.log2(), (self.sr / 2.).log2(), 0., 1.)
    }
}

impl View for FrequencyMarkers {
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
            let freq_text = format!("C{}", c_idx);

            let x_pos = self.freq_to_pos(*freq) * width;

            // let text_metrics = canvas.measure_text(0., 0., &freq_text, paint);

            // if let Ok(metrics) = text_metrics {
            //     canvas.fill_text(width - metrics.width(), n as f32*step_height, &vol_text, paint);
            // }

            path.move_to(x_pos, 0.);
            path.line_to(x_pos, height);
            canvas.fill_text(x_pos, height, &freq_text, text_paint);
        }

        canvas.stroke_path(&mut path, line_paint);
    }
}
