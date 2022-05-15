use std::sync::{Arc, Mutex};

use vizia::prelude::*;
use crate::ui::visualizer::Spectrometer;

use self::visualizer::{Style, Scale};

mod visualizer;

#[derive(Lens)]
pub struct UIData {
    data: Vec<f32>,
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|e, _| {
            match e {
                Events::Update(data) => {
                    self.data = data.clone();
                }
            }
        });
    }
}

pub enum Events {
    Update(Vec<f32>)
}

pub fn ui(mut delivery_mutex: Arc<Mutex<Vec<f32>>>, sampling_rate: usize) {
    
    Application::new(move |cx| {
        UIData {
            data: vec![0.0; crate::FFT_SIZE],
        }.build(cx);
        ZStack::new(cx, |cx| {
            // Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Root(0.2));
            // Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Root(0.5));
            // Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Root(0.6));
            // Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Linear);
            Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Logarithmic, vizia::vg::Color::hex("#f54e47"), 0.5, 0.);
            // Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Linear, vizia::vg::Color::hex("#00ff00"));
            // Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Logarithmic, vizia::vg::Color::white());
        });
        

    })
    .on_idle(move |cx| {
        if let Ok(x) = delivery_mutex.lock() {
            cx.emit(Events::Update(x.clone()));
        }
    })
    .background_color(Color::rgb(14, 11, 12))
    .run();
}

