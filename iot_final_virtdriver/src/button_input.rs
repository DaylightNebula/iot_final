#[derive(Default, Debug, Clone, Copy)]
pub struct ButtonInput {
    last: bool,
    current: bool,
    was_pressed: bool,
    was_released: bool
}

impl ButtonInput {
    pub fn update(&mut self, current: bool) {
        self.current = current;
        self.was_pressed = current != self.last && current;
        self.was_released = current != self.last && !current;
        self.last = current;
    }

    pub fn current(&self) -> bool { self.current }
    pub fn was_released(&self) -> bool { self.was_released }
    pub fn was_pressed(&self) -> bool { self.was_pressed }
}