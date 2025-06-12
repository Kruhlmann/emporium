use serde::Deserialize;

use crate::{CardTemplate, GlobalCardId};

use super::{Player, PlayerHealth};

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerTemplate {
    pub health: u64,
    #[serde(default)]
    pub regen: i64,
    #[serde(default, rename = "cards")]
    pub card_templates: Vec<CardTemplate>,
    #[serde(default, rename = "skills")]
    pub skill_templates: Vec<CardTemplate>,
}

impl PlayerTemplate {
    pub fn create_player(self, card_ids: Vec<GlobalCardId>) -> anyhow::Result<Player> {
        Ok(Player {
            health: PlayerHealth(self.health.try_into()?, self.health),
            shield_stacks: 0,
            poison_stacks: 0,
            burn_stacks: 0,
            regeneration_stacks: self.regen,
            dot_counter: 0,
            card_ids,
            template: self,
        })
    }
}
