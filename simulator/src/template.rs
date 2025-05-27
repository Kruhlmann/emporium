use serde::Deserialize;

use crate::PlayerTemplate;

#[derive(Clone, Debug, Deserialize)]
pub struct SimulationTemplate {
    pub player: PlayerTemplate,
    pub opponent: PlayerTemplate,
    #[serde(skip, default)]
    pub source: Option<String>,
}
