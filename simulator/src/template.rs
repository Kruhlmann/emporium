use serde::Deserialize;

use crate::PlayerTemplate;

#[derive(Clone, Debug, Deserialize)]
pub struct SimulationTemplate {
    pub player: PlayerTemplate,
    pub opponent: PlayerTemplate,
    #[serde(default)]
    pub seed: Option<u64>,
}

impl SimulationTemplate {
    pub fn invert(other: &SimulationTemplate) -> SimulationTemplate {
        let other = other.clone();
        SimulationTemplate {
            player: other.opponent,
            opponent: other.player,
            seed: other.seed,
        }
    }
}
