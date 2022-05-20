use vizia::prelude::Data;

#[derive(Clone, Copy, Data)]
pub struct Bin {
    val: f32,
    history: f32,
    frequency: f32,
    smooth_val: f32
}

impl Bin {
    pub fn new(val: f32) -> Self {
        Bin {val, 
            history: -90., 
            frequency: -90.,
            smooth_val: val
        }
    }

    pub fn update(&mut self, new_val: f32) {
        if self.history < new_val {
            self.history = new_val;
        }
        self.val = new_val;

        // TODO: Attack smoothing
        // Decay smoothing
        let res = self.val + 0.95 * (self.history - self.val);
        self.history = res;
        self.smooth_val = res;
    }

    pub fn get_smooth_val(&self) -> f32 {
        self.smooth_val
    }

    pub fn get_raw_val(&self) -> f32 {
        self.val
    }
}