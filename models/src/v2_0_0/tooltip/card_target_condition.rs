use crate::v2_0_0::{Card, Size, Tag};

use super::PlayerTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetCondition {
    Always,
    Never,
    IsSelf,
    OwnedByPlayer(PlayerTarget),
    HasTag(Tag),
    IsOfSize(Size),

    And(Box<TargetCondition>, Box<TargetCondition>),
    Or(Box<TargetCondition>, Box<TargetCondition>),
    Not(Box<TargetCondition>),
    Raw(String),
}

impl Card {
    pub fn matches(
        &self,
        condition: &TargetCondition,
        card_owner: PlayerTarget,
        target_candidate: &Card,
    ) -> bool {
        match condition {
            TargetCondition::Always => true,
            TargetCondition::Never => false,
            TargetCondition::IsSelf => *self == *target_candidate,
            TargetCondition::OwnedByPlayer(condition_owner) => card_owner == *condition_owner,
            TargetCondition::HasTag(tag) => self.tags.iter().find(|t| *t == tag).is_some(),
            TargetCondition::IsOfSize(size) => self.size == *size,
            TargetCondition::And(a, b) => {
                self.matches(a, card_owner, target_candidate)
                    && self.matches(b, card_owner, target_candidate)
            }
            TargetCondition::Or(a, b) => {
                self.matches(a, card_owner, target_candidate)
                    || self.matches(b, card_owner, target_candidate)
            }
            TargetCondition::Not(a) => !self.matches(a, card_owner, target_candidate),
            TargetCondition::Raw(s) => {
                eprintln!("skipping raw target condition: '{s}'");
                false
            }
        }
    }
}

impl std::ops::BitAnd for TargetCondition {
    type Output = TargetCondition;

    fn bitand(self, rhs: TargetCondition) -> TargetCondition {
        TargetCondition::And(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::BitOr for TargetCondition {
    type Output = TargetCondition;

    fn bitor(self, rhs: TargetCondition) -> TargetCondition {
        TargetCondition::Or(Box::new(self), Box::new(rhs))
    }
}

impl std::ops::Not for TargetCondition {
    type Output = TargetCondition;

    fn not(self) -> TargetCondition {
        TargetCondition::Not(Box::new(self))
    }
}

impl std::fmt::Display for TargetCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetCondition::And(i, j) => {
                write!(f, "TargetCondition::And(Box::new({i}), Box::new({j}))")
            }
            TargetCondition::Or(i, j) => {
                write!(f, "TargetCondition::Or(Box::new({i}), Box::new({j}))")
            }
            TargetCondition::HasTag(i) => {
                write!(f, "TargetCondition::HasTag(Tag::{i:?})")
            }
            TargetCondition::Raw(i) => {
                write!(f, "TargetCondition::Raw({i:?}.to_string())")
            }
            TargetCondition::Always => write!(f, "TargetCondition::Always"),
            TargetCondition::Never => write!(f, "TargetCondition::Never"),
            TargetCondition::IsSelf => write!(f, "TargetCondition::IsSelf"),
            TargetCondition::OwnedByPlayer(i) => {
                write!(f, "TargetCondition::OwnedByPlayer(PlayerTarget::{i:?})")
            }
            TargetCondition::IsOfSize(i) => write!(f, "TargetCondition::IsOfSize(Size::{i:?})"),
            TargetCondition::Not(i) => write!(f, "TargetCondition::Not({i})"),
        }
    }
}

impl TargetCondition {
    pub fn from_str(s: &str) -> Self {
        match s.replace(".", "").to_lowercase().trim() {
            "a property" => TargetCondition::HasTag(Tag::Property),
            "a core" => TargetCondition::HasTag(Tag::Core),
            "a friend" => TargetCondition::HasTag(Tag::Core),
            "all your other items" => TargetCondition::Always,
            s => TargetCondition::Raw(s.to_string()),
        }
    }
}
