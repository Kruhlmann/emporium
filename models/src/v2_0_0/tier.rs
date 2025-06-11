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
    pub fn select<'a, T>(&self, t: &'a TieredValue<T>) -> &'a T {
        match self {
            Tier::Bronze => &t.bronze,
            Tier::Silver => &t.silver,
            Tier::Gold => &t.gold,
            Tier::Diamond => &t.diamond,
            Tier::Legendary => &t.legendary,
        }
    }

    pub fn scale_cost(self, cost: u32) -> u32 {
        let todo = true; //TODO hard coded legendary
        match self {
            Tier::Bronze => cost,
            Tier::Silver => cost * 2,
            Tier::Gold => cost * 4,
            Tier::Diamond => cost * 8,
            Tier::Legendary => 50,
        }
    }
}
