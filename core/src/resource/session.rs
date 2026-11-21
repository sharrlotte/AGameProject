use bevy::prelude::*;

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct SessionClock {
    accumulated_secs: f32,
    started_at: Option<f32>,
}

impl SessionClock {
    pub fn resume(&mut self, now: f32, saved_secs: f32) {
        self.accumulated_secs = saved_secs;
        self.started_at = Some(now);
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn elapsed(&self, now: f32) -> f32 {
        self.accumulated_secs
            + self
                .started_at
                .map_or(0.0, |started| (now - started).max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulated_time_survives_resume() {
        let mut clock = SessionClock::default();
        clock.resume(10.0, 0.0);
        assert_eq!(clock.elapsed(25.0), 15.0);

        clock.resume(30.0, clock.elapsed(30.0));
        assert_eq!(clock.elapsed(40.0), 30.0);
    }

    #[test]
    fn reset_clears_accumulated_time() {
        let mut clock = SessionClock::default();
        clock.resume(0.0, 100.0);
        clock.reset();
        assert_eq!(clock.elapsed(50.0), 0.0);
    }
}
