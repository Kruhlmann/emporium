use std::time::Duration;

use crate::{CombatEvent, Player, SimulationDrawType, TaggedCombatEvent};

#[derive(Debug)]
pub struct SimulationResultInner {
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
        let mut res = match self {
            SimulationResult::Victory(..) => {
                format!("[{:?}] Victory\n", inner.duration)
            }
            SimulationResult::Defeat(..) => format!("[{:?}] Defeat\n", inner.duration),
            SimulationResult::Draw(SimulationDrawType::Timeout, ..) => {
                format!("[{:?}] Draw by timeout", inner.duration)
            }
            SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, ..) => {
                format!("[{:?}] Draw by simultaneous defeat\n", inner.duration,)
            }
        };
        let events = match self {
            SimulationResult::Victory(i) => &i.events,
            SimulationResult::Defeat(i) => &i.events,
            SimulationResult::Draw(.., i) => &i.events,
        };
        let mut last_line_was_tick = false;
        for event in events {
            match &event.1 {
                CombatEvent::Tick(n) => {
                    if !last_line_was_tick {
                        res.push_str(&format!("[{n}] Tick\n"));
                        last_line_was_tick = true;
                    }
                }
                ref e => {
                    last_line_was_tick = false;
                    res.push_str(&format!("  {e:?} -> {:?}\n", event.0))
                }
            }
        }
        write!(f, "{res}")
    }
}
