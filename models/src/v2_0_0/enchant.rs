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

impl std::fmt::Display for CardEnchantment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CardEnchantment::{self:?}")
    }
}

#[derive(Debug, Clone, Deserialize)]
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
