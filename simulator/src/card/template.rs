use std::time::Duration;

use models::v2_0_0::{Effect, EffectEvent, Modifier, PlayerTarget, Tier, Tooltip};
use serde::Deserialize;

use crate::GameTicks;

use super::{Card, CardModification, GlobalCardId};

#[derive(Clone, Debug, Deserialize)]
pub struct CardTemplate {
    pub name: String,
    pub tier: Tier,
    #[serde(default)]
    pub modifications: Vec<CardModification>,
}

impl CardTemplate {
    pub fn create_card_on_board(
        &self,
        position: u8,
        owner: PlayerTarget,
        id: GlobalCardId,
    ) -> anyhow::Result<Card> {
        let create_item: fn() -> models::v2_0_0::Card =
            *gamedata::v2_0_0::cards::CONSTRUCT_CARD_BY_NAME
                .get(self.name.as_str())
                .ok_or(anyhow::anyhow!("unknown card {:?}", &self.name))?;
        let inner = create_item();
        let tooltips = self.tier.select(inner.tiers.clone());
        if tooltips.len() == 0 {
            anyhow::bail!("no tooltips on card {} of tier {:?}", self.name, self.tier);
        }
        let cooldown_effects: Vec<Effect> = tooltips
            .iter()
            .flat_map(|t| match t {
                Tooltip::When(EffectEvent::OnCooldown(e)) => Some(e.clone()),
                _ => None,
            })
            .collect();
        let cooldown = tooltips
            .iter()
            .find_map(|t| match t {
                Tooltip::StaticModifier(Modifier::Cooldown(c)) => {
                    Some(Duration::from_millis((c * 1000.0).round() as u64))
                }
                _ => None,
            })
            .unwrap_or(Duration::from_secs(0));

        Ok(Card {
            inner,
            position,
            owner,
            tier: self.tier,
            id_for_simulation: id,
            modifications: self.modifications.clone(),
            cooldown: cooldown.into(),
            cooldown_effects,
            cooldown_counter: 0,
            freeze_ticks: GameTicks::default(),
            slow_ticks: GameTicks::default(),
        })
    }
}
