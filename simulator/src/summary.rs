use std::time::Duration;

use crate::{SimulationDrawType, SimulationResult};

#[derive(Debug, Clone)]
pub struct SimulationSummary {
    pub total_runs: usize,
    pub victories: usize,
    pub defeats: usize,
    pub draw_timeout: usize,
    pub draw_simultaneous: usize,
    pub average_duration: Duration,
    pub average_player_health: f32,
    pub average_opponent_health: f32,
}

impl From<&Vec<SimulationResult>> for SimulationSummary {
    fn from(results: &Vec<SimulationResult>) -> Self {
        let total_runs = results.len();
        let mut victories = 0;
        let mut defeats = 0;
        let mut draw_timeout = 0;
        let mut draw_simultaneous = 0;
        let mut sum_duration = Duration::ZERO;
        let mut sum_player_health = 0f64;
        let mut sum_opponent_health = 0f64;

        for res in results.iter() {
            match res {
                SimulationResult::Victory(..) => victories += 1,
                SimulationResult::Defeat(..) => defeats += 1,
                SimulationResult::Draw(kind, ..) => match kind {
                    SimulationDrawType::Timeout => draw_timeout += 1,
                    SimulationDrawType::SimultaneousDefeat => draw_simultaneous += 1,
                },
            }
            let inner = res.inner_ref();
            sum_duration += inner.duration;
            sum_player_health += inner.player.health.current() as f64;
            sum_opponent_health += inner.opponent.health.current() as f64;
        }

        let average_duration = if total_runs > 0 {
            sum_duration / (total_runs as u32)
        } else {
            Duration::ZERO
        };

        let average_player_health = if total_runs > 0 {
            (sum_player_health / total_runs as f64) as f32
        } else {
            0.0
        };

        let average_opponent_health = if total_runs > 0 {
            (sum_opponent_health / total_runs as f64) as f32
        } else {
            0.0
        };

        SimulationSummary {
            total_runs,
            victories,
            defeats,
            draw_timeout,
            draw_simultaneous,
            average_duration,
            average_player_health,
            average_opponent_health,
        }
    }
}
