use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TieredValue<T> {
    pub bronze: T,
    pub silver: T,
    pub gold: T,
    pub diamond: T,
    pub legendary: T,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
    Diamond,
    Legendary,
}

impl Tier {
    pub fn select<T>(&self, t: TieredValue<T>) -> T {
        match self {
            Tier::Bronze => t.bronze,
            Tier::Silver => t.silver,
            Tier::Gold => t.gold,
            Tier::Diamond => t.diamond,
            Tier::Legendary => t.legendary,
        }
    }
}
