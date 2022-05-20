use vizia::prelude::Data;

#[derive(Clone, Copy, Data)]
pub struct Bin {
    val: f32,
    history: f32,
    attack: f32,
    release: f32,
    frequency: f32,
    smooth_val: f32,
}

impl Bin {
    pub fn new(val: f32) -> Self {
        Bin {
            val,
            history: -90.,
            attack: 0.5,
            release: 0.95,
            frequency: 0.,
            smooth_val: val,
        }
    }

    pub fn update(&mut self, new_val: f32) {
        let direction_strength = if self.history > new_val {
            self.release
        } else {
            self.attack
        };

        self.history = (self.history * direction_strength) + (new_val * (1. - direction_strength));
        self.smooth_val = self.history;
    }

    pub fn get_smooth_val(&self) -> f32 {
        self.smooth_val
    }

    pub fn get_raw_val(&self) -> f32 {
        self.val
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }
}
