use super::Animation;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Animator {
    animation: Animation,
    current_step: usize,
    current_frame: usize,
    tick_timer: u16,
}

impl Animator {
    pub fn new(animation: Animation) -> Animator {
        Animator {
            animation,
            current_step: 0,
            current_frame: 0,
            tick_timer: 0,
        }
    }
    pub fn reset(&mut self) {
        self.current_step = 0;
        self.current_frame = 0;
        self.tick_timer = 0;
    }
    pub fn tick(&mut self) -> bool {
        if let Some(tick_max) = self.animation.steps()[self.current_step].frames()[self.current_frame].ticks {
            self.tick_timer += 1;
            if self.tick_timer >= tick_max {
                self.tick_timer = 0;
                self.current_frame += 1;
                if self.current_frame >= self.animation.steps()[self.current_step].frames().len() {
                    let (start_step, start_frame) = {
                        if let Some(looping_frame_info) = self.animation.looping_frame() {
                            looping_frame_info
                        }
                        else {
                            (0, 0)
                        }
                    };
                    self.current_step += 1;
                    if self.current_step >= self.animation.steps().len() {
                        self.current_step = start_step;
                        self.current_frame = start_frame;
                    }
                    else {
                        self.current_frame = 0;
                    }
                }
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }
    pub fn animation(&self) -> &Animation {
        &self.animation
    }
    pub fn current_display_info(&self) -> (u16, u16) {
        let current_frame = &self.animation.steps()[self.current_step].frames()[self.current_frame];
        (current_frame.group, current_frame.image)
    }
}
