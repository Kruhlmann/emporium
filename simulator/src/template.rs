use serde::Deserialize;

use crate::PlayerTemplate;

#[derive(Clone, Debug, Deserialize)]
pub struct SimulationTemplate {
    pub player: PlayerTemplate,
    pub opponent: PlayerTemplate,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(skip, default)]
    pub source: Option<String>,
}
