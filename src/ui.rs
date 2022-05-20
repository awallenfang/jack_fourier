use std::sync::{Arc, Mutex};

use crate::ui::{
    frequency_markers::FrequencyMarkers, spectrometer::Spectrometer, volume_markers::VolumeMarkers,
};
use vizia::prelude::*;

use self::spectrometer::{Scale, Style};

pub(crate) mod bin;
mod frequency_markers;
mod spectrometer;
mod volume_markers;

#[derive(Lens)]
pub struct UIData {
    data: Vec<f32>,
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|e, _| match e {
            Events::Update(data) => {
                self.data = data.clone();
            }
        });
    }
}

pub enum Events {
    Update(Vec<f32>),
}

pub fn ui(delivery_mutex: Arc<Mutex<Vec<f32>>>, sampling_rate: usize) {
    Application::new(move |cx| {
        UIData {
            data: vec![-90.; crate::FFT_SIZE],
        }
        .build(cx);
        ZStack::new(cx, |cx| {
            FrequencyMarkers::new(cx, sampling_rate);
            VolumeMarkers::new(cx);
            Spectrometer::new(
                cx,
                UIData::data,
                sampling_rate,
                Style::Spectrum,
                Scale::Logarithmic,
                vizia::vg::Color::hex("#f54e47"),
                0.4,
                3.,
            );
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
