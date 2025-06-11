use models::v2_0_0::{Modifier, Percentage, Tooltip};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub enum CardModification {
    Enchanted(models::v2_0_0::Enchantment),
    Value(u32),
    Burn(u32),
    Poison(u32),
    Shield(u32),
    Heal(u32),
    Damage(u32),
    Cooldown(Percentage),
    Crit(Percentage),
}

impl CardModification {
    pub fn derive_tooltips(&self, card: &models::v2_0_0::Card) -> Vec<Tooltip> {
        match self {
            CardModification::Enchanted(e) => card
                .enchantments
                .iter()
                .find(|en| en.is_enchantment(e))
                .map(|e| e.tooltips())
                .unwrap_or(Vec::new()),
            CardModification::Value(value) => {
                vec![Tooltip::StaticModifier(Modifier::IncreasedValue(*value))]
            }
            CardModification::Burn(_) => todo!(),
            CardModification::Poison(_) => todo!(),
            CardModification::Shield(_) => todo!(),
            CardModification::Heal(_) => todo!(),
            CardModification::Damage(_) => todo!(),
            CardModification::Cooldown(..) => todo!(),
            CardModification::Crit(..) => todo!(),
        }
    }
}
