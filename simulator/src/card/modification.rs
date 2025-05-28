use std::rc::Rc;

use models::v2_0_0::{Percentage, Tooltip};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub enum CardModification {
    Enchanted(models::v2_0_0::Enchantment),
    Value(u64),
    Burn(u64),
    Poison(u64),
    Shield(u64),
    Heal(u64),
    Damage(u64),
    Cooldown(Percentage),
    Crit(Percentage),
}

impl CardModification {
    pub fn derive_tooltips<'a>(&'a self, card: &'a models::v2_0_0::Card) -> Rc<&'a [Tooltip]> {
        match self {
            CardModification::Enchanted(e) => card
                .enchantments
                .iter()
                .find(|en| en.is_enchantment(e))
                .map(|e| e.tooltips())
                .map(Rc::new)
                .unwrap_or(Rc::new(&[])),
            CardModification::Value(_) => todo!(),
            CardModification::Burn(_) => todo!(),
            CardModification::Poison(_) => todo!(),
            CardModification::Shield(_) => todo!(),
            CardModification::Heal(_) => todo!(),
            CardModification::Damage(_) => todo!(),
            CardModification::Cooldown(percentage) => todo!(),
            CardModification::Crit(percentage) => todo!(),
        }
    }
}
