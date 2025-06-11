use serde::Deserialize;

use super::Tooltip;

#[derive(Debug, Clone, PartialEq)]
pub enum CardEnchantment {
    Heavy(Vec<Tooltip>),
    Icy(Vec<Tooltip>),
    Turbo(Vec<Tooltip>),
    Shielded(Vec<Tooltip>),
    Restorative(Vec<Tooltip>),
    Toxic(Vec<Tooltip>),
    Fiery(Vec<Tooltip>),
    Shiny(Vec<Tooltip>),
    Deadly(Vec<Tooltip>),
    Radiant(Vec<Tooltip>),
    Obsidian(Vec<Tooltip>),
    Golden(Vec<Tooltip>),
}

impl CardEnchantment {
    pub fn is_enchantment(&self, e: &Enchantment) -> bool {
        match self {
            CardEnchantment::Heavy(..) => *e == Enchantment::Heavy,
            CardEnchantment::Icy(..) => *e == Enchantment::Icy,
            CardEnchantment::Turbo(..) => *e == Enchantment::Turbo,
            CardEnchantment::Shielded(..) => *e == Enchantment::Shielded,
            CardEnchantment::Restorative(..) => *e == Enchantment::Restorative,
            CardEnchantment::Toxic(..) => *e == Enchantment::Toxic,
            CardEnchantment::Fiery(..) => *e == Enchantment::Fiery,
            CardEnchantment::Shiny(..) => *e == Enchantment::Shiny,
            CardEnchantment::Deadly(..) => *e == Enchantment::Deadly,
            CardEnchantment::Radiant(..) => *e == Enchantment::Radiant,
            CardEnchantment::Obsidian(..) => *e == Enchantment::Obsidian,
            CardEnchantment::Golden(..) => *e == Enchantment::Golden,
        }
    }

    pub fn tooltips(&self) -> Vec<Tooltip> {
        match self {
            CardEnchantment::Heavy(ts)
            | CardEnchantment::Icy(ts)
            | CardEnchantment::Turbo(ts)
            | CardEnchantment::Shielded(ts)
            | CardEnchantment::Restorative(ts)
            | CardEnchantment::Toxic(ts)
            | CardEnchantment::Fiery(ts)
            | CardEnchantment::Shiny(ts)
            | CardEnchantment::Deadly(ts)
            | CardEnchantment::Radiant(ts)
            | CardEnchantment::Obsidian(ts)
            | CardEnchantment::Golden(ts) => ts.to_vec(),
        }
    }
}

impl std::fmt::Display for CardEnchantment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CardEnchantment::{self:?}")
    }
}

#[derive(Copy, Debug, Clone, Deserialize, PartialEq)]
pub enum Enchantment {
    Heavy,
    Icy,
    Turbo,
    Shielded,
    Restorative,
    Toxic,
    Fiery,
    Shiny,
    Deadly,
    Radiant,
    Obsidian,
    Golden,
}
