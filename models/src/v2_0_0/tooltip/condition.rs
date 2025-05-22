use crate::v2_0_0::{Size, Tag};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    HasCardOfSize(Size),
    HasCardOfTag(Tag),
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::HasCardOfSize(i) => {
                write!(f, "Condition::HasCardOfSize(Size::{i:?})")
            }
            Condition::HasCardOfTag(i) => {
                write!(f, "Condition::HasCardOfTag(Tag::{i:?})")
            }
        }
    }
}
