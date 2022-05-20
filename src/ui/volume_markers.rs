use vizia::prelude::*;
use vizia::vg::{Paint, Path};
pub struct VolumeMarkers {
    min: f32,
    max: f32,
    stops: i32
}

impl VolumeMarkers {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            min: -90.,
            max: 0.,
            stops: 7
        }.build(cx, |_cx| {})
    }
}

impl View for VolumeMarkers {
    fn draw(&self, cx: &mut DrawContext<'_>, canvas: &mut Canvas) {
        let entity = cx.current();

        let bounds = cx.cache().get_bounds(entity);

        //Skip meters with no width or no height
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let width = bounds.w;
        let height = bounds.h;

        // One extra step, so we can have 5 steps between the top and bottom
        let step_db_size = (self.max - self.min) / (self.stops + 1) as f32;
        let step_height = height / (self.stops + 1) as f32;

        let line_paint = Paint::color(vizia::vg::Color::hex("#565454"));

        let text_paint = Paint::color(vizia::vg::Color::white());
        
        let mut path = Path::new();
        let mut volume_db = 0.;
        for n in 1..=self.stops {
            volume_db += step_db_size;
            let vol_text = format!("-{}dB", volume_db);

            let text_metrics = canvas.measure_text(0., 0., &vol_text, text_paint);
            
            if let Ok(metrics) = text_metrics {
                canvas.fill_text(width - metrics.width(), n as f32*step_height, &vol_text, text_paint);
            }

            path.move_to(0., n as f32*step_height);
            path.line_to(width, n as f32*step_height);
        }

        canvas.stroke_path(&mut path, line_paint);
    }
}