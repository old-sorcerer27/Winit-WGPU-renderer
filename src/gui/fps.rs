use std::collections::VecDeque;

struct FpsCounter {
    frame_times: VecDeque<f32>,
}

impl FpsCounter {
    const MAX_FRAME_TIMES: usize = 8;

    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(Self::MAX_FRAME_TIMES),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_times.push_back(dt);
        if self.frame_times.len() > Self::MAX_FRAME_TIMES {
            self.frame_times.pop_front();
        }
    }

    pub fn average_fps(&self) -> f32 {
        let sum: f32 = self.frame_times.iter().sum();
        self.frame_times.len() as f32 / sum
    }
}