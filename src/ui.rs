use std::sync::{Arc, Mutex};

use crate::ui::{
    frequency_markers::FrequencyMarkers, spectrometer::Spectrometer, volume_markers::VolumeMarkers,
};
use vizia::prelude::*;

use self::{spectrometer::{Scale, Style, SpectrometerHandle}, frequency_markers::FreqMarkerHandle};

pub(crate) mod bin;
mod frequency_markers;
mod spectrometer;
mod volume_markers;

const STYLE: &str = r#"
    label {
        font-size: 20;
        color: #C2C2C2;
        left: 1s;
        right: 1s;
    }
    
    knob {
        width: 100px;
        height: 100px;
        left: 1s;
        right: 1s;
    }
    
    knob .track {
        background-color: #ffb74d;
    }
    .label_knob {
        border-width: 2px;
        border-color: #28282b;
        background-color: #000000;
        col-between: 10px;
        child-space: 1s;
    }
"#;

#[derive(Lens)]
pub struct UIData {
    data: Vec<f32>,
    attack: f32,
    release: f32,
    sr: usize,
    min_freq: f32,
    max_freq: f32
}

impl Model for UIData {
    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        event.map(|e, _| match e {
            Events::Update(data) => {
                self.data = data.clone();
            }
            Events::AttackChange(x) => {
                self.attack = *x;
            }
            Events::ReleaseChange(x) => {
                self.release = *x;
            }
            Events::MinChange(x) => {
                self.min_freq = *x;
            },
            Events::MaxChange(x) => {
                self.max_freq = *x;
            },
        });
    }
}

pub enum Events {
    Update(Vec<f32>),
    AttackChange(f32),
    ReleaseChange(f32),
    MinChange(f32),
    MaxChange(f32)
}

pub fn ui(delivery_mutex: Arc<Mutex<Vec<f32>>>, sampling_rate: usize) {
    Application::new(move |cx| {
        UIData {
            data: vec![-90.; crate::FFT_SIZE],
            attack: 0.5,
            release: 0.9,
            sr: sampling_rate,
            min_freq: 0.,
            max_freq: 1.
        }
        .build(cx);

        cx.add_theme(STYLE);

        // TODO: Add knobs to connect attack and release over lenses
        VStack::new(cx, |cx| {
            ZStack::new(cx, |cx| {
                FrequencyMarkers::new(cx, sampling_rate)
                .min(UIData::min_freq)
                .max(UIData::max_freq);

                VolumeMarkers::new(cx);

                Spectrometer::new(
                    cx,
                    UIData::data,
                    sampling_rate,
                    Style::Spectrum,
                    Scale::Logarithmic,
                    vizia::vg::Color::hex("#f54e47"),
                )
                .attack(UIData::attack)
                .release(UIData::release)
                .min(UIData::min_freq)
                .max(UIData::max_freq);
            })
            .height(Percentage(80.));
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Knob::new(cx, 0., UIData::min_freq, false)
                    .on_changing(move |cx, val| cx.emit(Events::MinChange(val)));
                    Label::new(cx, "Min Hz");
                });
                VStack::new(cx, |cx| {
                    Knob::new(cx, 1., UIData::max_freq, false)
                    .on_changing(move |cx, val| cx.emit(Events::MaxChange(val)));
                    Label::new(cx, "Max Hz");
                });
                VStack::new(cx, |cx| {
                    Knob::new(cx, 0.5, UIData::attack, false)
                    .on_changing(move |cx, val| cx.emit(Events::AttackChange(val)));
                    Label::new(cx, "Attack");
                });
                VStack::new(cx, |cx| {
                    Knob::new(cx, 0.9, UIData::release, false)
                    .on_changing(move |cx, val| cx.emit(Events::ReleaseChange(val)));
                    Label::new(cx, "Release");
                });
            });
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
