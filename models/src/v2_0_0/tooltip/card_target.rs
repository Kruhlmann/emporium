use crate::v2_0_0::Size;

use super::TargetCondition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardTarget {
    This,
    Adjacent,
    NOfSize(usize, Size),
    NOfSizeOpponent(usize, Size),
    NOwn(usize),
    NOpponent(usize),
    Own,
    Opponent,
    OtherWeapons,
    Conditional(TargetCondition),
    NameIncludesString(String),
}

impl std::fmt::Display for CardTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardTarget::This => write!(f, "CardTarget::This"),
            CardTarget::Adjacent => write!(f, "CardTarget::Adjacent"),
            CardTarget::NOfSize(n, i) => write!(f, "CardTarget::NOfSize({n}, Size::{i:?})"),
            CardTarget::NOfSizeOpponent(n, i) => {
                write!(f, "CardTarget::NOfSizeOpponent({n}, Size::{i:?})")
            }
            CardTarget::Own => write!(f, "CardTarget::Own"),
            CardTarget::NOwn(i) => write!(f, "CardTarget::NOwn({i})"),
            CardTarget::NOpponent(i) => write!(f, "CardTarget::NOpponent({i})"),
            CardTarget::Opponent => write!(f, "CardTarget::Opponent"),
            CardTarget::OtherWeapons => write!(f, "CardTarget::OtherWeapons"),
            CardTarget::Conditional(i) => write!(f, "CardTarget::Conditional({i})"),
            CardTarget::NameIncludesString(i) => {
                write!(f, "CardTarget::NameIncludesString({i:?}.to_string())")
            }
        }
    }
}
