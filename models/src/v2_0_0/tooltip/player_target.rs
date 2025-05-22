use serde::Deserialize;

#[derive(Copy, PartialEq, Eq, Deserialize, Debug, Clone)]
pub enum PlayerTarget {
    Player,
    Opponent,
}

impl PlayerTarget {
    pub fn inverse(&self) -> PlayerTarget {
        match self {
            PlayerTarget::Player => PlayerTarget::Opponent,
            PlayerTarget::Opponent => PlayerTarget::Player,
        }
    }
}

impl std::fmt::Display for PlayerTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerTarget::Player => write!(f, "PlayerTarget::Player"),
            PlayerTarget::Opponent => write!(f, "PlayerTarget::Opponent"),
        }
    }
}
