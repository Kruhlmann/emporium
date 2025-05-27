use std::time::Duration;

lazy_static::lazy_static! {
    pub static ref NUMBER_OF_BOARD_SPACES: u8 = 10;
    pub static ref TICKS_PER_SECOND: usize = 60;
    pub static ref TICKRATE: f32 = 1000.0 / *TICKS_PER_SECOND as f32;
    pub static ref TICK_DURATION: Duration = Duration::from_millis(TICKRATE.round() as u64);
    pub static ref DURATION_BEFORE_SANDSTORM: Duration = Duration::from_secs(35);
    pub static ref MAX_FIGHT_DURATION: Duration = Duration::from_secs(300);
    pub static ref SIMULATION_TICK_COUNT: usize = {
        let fight_ms = MAX_FIGHT_DURATION.as_micros();
        let tick_ms  = TICK_DURATION.as_micros();
        (fight_ms / tick_ms) as usize
    };
}
