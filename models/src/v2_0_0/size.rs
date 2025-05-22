#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    pub fn board_spaces(&self) -> u8 {
        match self {
            Size::Small => 1,
            Size::Medium => 2,
            Size::Large => 3,
        }
    }
}

impl TryFrom<&str> for Size {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "small" => Ok(Size::Small),
            "medium" => Ok(Size::Medium),
            "large" => Ok(Size::Large),
            size => anyhow::bail!("invalid size {size}"),
        }
    }
}
