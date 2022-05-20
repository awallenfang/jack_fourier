use std::sync::{Arc, Mutex};

use vizia::prelude::*;
use crate::{ui::{spectrometer::Spectrometer, frequency_markers::FrequencyMarkers, volume_markers::VolumeMarkers}};

use self::spectrometer::{Style, Scale};

mod spectrometer;
mod frequency_markers;
mod volume_markers;
pub(crate) mod bin;

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
            data: vec![-90.; crate::FFT_SIZE],
        }.build(cx);
        ZStack::new(cx, |cx| {
            FrequencyMarkers::new(cx, sampling_rate);
            VolumeMarkers::new(cx);
            Spectrometer::new(cx, UIData::data, sampling_rate, Style::Spectrum, Scale::Logarithmic, vizia::vg::Color::hex("#f54e47"), 0.4, 3.);
            
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

