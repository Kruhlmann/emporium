use std::time::Duration;

use crate::{Player, SimulationDrawType, TaggedCombatEvent};

#[derive(Debug)]
pub struct SimulationResultInner {
    pub source: Option<String>,
    pub events: Vec<TaggedCombatEvent>,
    pub duration: Duration,
    pub player: Player,
    pub opponent: Player,
}

#[derive(Debug)]
pub enum SimulationResult {
    Victory(SimulationResultInner),
    Defeat(SimulationResultInner),
    Draw(SimulationDrawType, SimulationResultInner),
}

impl SimulationResult {
    pub fn inner_ref(&self) -> &SimulationResultInner {
        match self {
            SimulationResult::Victory(r)
            | SimulationResult::Defeat(r)
            | SimulationResult::Draw(SimulationDrawType::Timeout, r)
            | SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, r) => r,
        }
    }

    pub fn short_str(&self) -> String {
        match self {
            SimulationResult::Victory(..) => "Victory".to_string(),
            SimulationResult::Defeat(..) => "Defeat".to_string(),
            SimulationResult::Draw(draw_type, ..) => format!("Draw ({draw_type:?})"),
        }
    }
}

impl std::fmt::Display for SimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner_ref();
        let source_str = inner
            .source
            .as_ref()
            .map(|s| format!(" [{s}] "))
            .unwrap_or("".to_string());
        let header = match self {
            SimulationResult::Victory(..) => {
                format!("[{:?}]{source_str} Victory", inner.duration)
            }
            SimulationResult::Defeat(..) => format!("[{:?}]{source_str} Defeat", inner.duration),
            SimulationResult::Draw(SimulationDrawType::Timeout, ..) => {
                format!("[{:?}]{source_str} Draw by timeout", inner.duration)
            }
            SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, ..) => {
                format!(
                    "[{:?}]{source_str} Draw by simultaneous defeat",
                    inner.duration,
                )
            }
        };
        write!(f, "{header}")
    }
}
