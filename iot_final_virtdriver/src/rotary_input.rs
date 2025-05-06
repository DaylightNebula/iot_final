use uinput::{Device, Event};

pub struct RotaryInput {
    target: RotaryInputTarget,
    initial: Option<i16>,
    last: f32,
    current: f32
}

pub enum RotaryInputTarget {
    Button {
        button: Event,
        cross_value: f32 
    },
    Axis {
        axis: Event,
        flip: bool
    }
}

impl RotaryInput {
    pub fn new(target: RotaryInputTarget, default_position: f32) -> Self {
        Self {
            target,
            initial: None,
            last: default_position,
            current: default_position
        }
    }

    pub fn update(&mut self, device: &mut Device, input: i16) {
        // get or set initial value
        let initial = if let Some(initial) = self.initial { initial } else {
            self.initial = Some(input);
            input
        };

        // calculate current via difference / 2
        let current = f32::min(i16::abs(input - initial) as f32 / 1.0, 1.0);
        self.current = current;

        // trigger / update target here
        match &self.target {
            RotaryInputTarget::Button { button, cross_value } => {
                let a = current > *cross_value;
                let b = self.last > *cross_value;
                if a != b {
                    let _ = device.send(*button, if a { 255 } else { 0 });
                }
            },
            RotaryInputTarget::Axis { axis, flip } => {
                if current != self.last { 
                    let mut value = i32::abs((current * 255.0) as i32);
                    if *flip { value = 255 - value; }
                    let _ = device.send(*axis, value);
                }
            }
        }

        // update last chcker
        self.last = current;
    }

    pub fn get_cur(&self) -> f32 { self.current }
}