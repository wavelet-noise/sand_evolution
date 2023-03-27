pub struct FpsMeter {
    last_update_timestamp: f64,
    last_value: usize,
    count: usize,
}

impl FpsMeter {
    pub fn new() -> Self {
        Self {
            last_update_timestamp: instant::now(),
            last_value: 0,
            count: 0,
        }
    }
    pub fn next(&mut self) -> usize {
        if instant::now() - self.last_update_timestamp < 1000. {
            self.count += 1;
        } else {
            self.last_update_timestamp = instant::now();
            self.last_value = self.count;
            self.count = 0;
        }
        return self.last_value;
    }
}
