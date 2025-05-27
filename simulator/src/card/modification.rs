use models::v2_0_0::Percentage;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub enum CardModification {
    Enchanted(models::v2_0_0::Enchantment),
    Value(u64),
    Burn(u64),
    Posison(u64),
    Shield(u64),
    Heal(u64),
    Damage(u64),
    Cooldown(Percentage),
    Crit(Percentage),
}
