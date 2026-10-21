pub(crate) struct IgnoreValue {
    ignore_frames: u8,
    old_value: u8,
}

impl IgnoreValue {
    const IGNORE_FRAMES: u8 = 2;

    pub(crate) fn new() -> Self {
        Self {
            ignore_frames: 0,
            old_value: 0,
        }
    }

    pub(crate) fn value_with_ignore(&mut self, current_value: u8) -> u8 {
        if self.ignore_frames > 0 {
            self.ignore_frames -= 1;

            return self.old_value;
        }

        current_value
    }

    pub(crate) fn begin_ignore(&mut self, current_value: u8) {
        self.ignore_frames = Self::IGNORE_FRAMES;
        self.old_value = current_value;
    }
}
