use models::v2_0_0::PlayerTarget;

use crate::{Card, GameTicks, GlobalCardId};

#[derive(Clone, Debug)]
pub struct CardSummary {
    pub id: GlobalCardId,
    pub name: String,
    pub owner: PlayerTarget,
}

impl std::fmt::Display for CardSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Card<id={}, name={}, owner={}>",
            self.id, self.name, self.owner
        )
    }
}

impl From<&Card> for CardSummary {
    fn from(value: &Card) -> Self {
        Self {
            id: value.id_for_simulation,
            name: value.inner.name.to_string(),
            owner: value.owner,
        }
    }
}

impl From<&mut Card> for CardSummary {
    fn from(value: &mut Card) -> Self {
        Self {
            id: value.id_for_simulation,
            name: value.inner.name.to_string(),
            owner: value.owner,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DispatchableEvent {
    Log(String),
    Error(String),
    Warning(String),
    Tick,
    CardFrozen(CardSummary, GameTicks),
    CardHasted(CardSummary, GameTicks),
    CardSlowed(CardSummary, GameTicks),
}
