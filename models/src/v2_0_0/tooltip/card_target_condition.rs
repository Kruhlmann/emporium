use crate::v2_0_0::{Size, Tag};

use super::PlayerTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetCondition {
    Always,
    Never,
    IsSelf,
    Adjacent,
    HasCooldown,
    HasOwner(PlayerTarget),
    HasTag(Tag),
    HasSize(Size),
    NameIncludes(String),

    And(Box<TargetCondition>, Box<TargetCondition>),
    Or(Box<TargetCondition>, Box<TargetCondition>),
    Not(Box<TargetCondition>),
    Raw(String),
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
            TargetCondition::Adjacent => write!(f, "TargetCondition::Adjacent"),
            TargetCondition::HasCooldown => write!(f, "TargetCondition::HasCooldown"),
            TargetCondition::Always => write!(f, "TargetCondition::Always"),
            TargetCondition::Never => write!(f, "TargetCondition::Never"),
            TargetCondition::IsSelf => write!(f, "TargetCondition::IsSelf"),
            TargetCondition::HasOwner(i) => {
                write!(f, "TargetCondition::HasOwner(PlayerTarget::{i:?})")
            }
            TargetCondition::HasSize(i) => write!(f, "TargetCondition::HasSize(Size::{i:?})"),
            TargetCondition::Not(i) => write!(f, "TargetCondition::Not({i})"),
            TargetCondition::NameIncludes(i) => {
                write!(f, "TargetCondition::NameIncludes({i:?}.to_string())")
            }
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
