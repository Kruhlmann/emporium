#[derive(Debug, Clone, PartialEq)]
pub enum GlobalEvent {
    PlayerFallsBelowHpPercentage(f64),
}

impl std::fmt::Display for GlobalEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalEvent::PlayerFallsBelowHpPercentage(i) => {
                write!(f, "GlobalEvent::PlayerFallsBelowHpPercentage({i:.2})")
            }
        }
    }
}

impl AsRef<GlobalEvent> for GlobalEvent {
    fn as_ref(&self) -> &GlobalEvent {
        &self
    }
}
