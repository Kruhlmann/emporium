use serde::Deserialize;

#[derive(Clone, Debug, Copy, Deserialize)]
#[serde(transparent)]
#[derive(PartialOrd, PartialEq)]
pub struct Percentage(pub f64);

impl Percentage {
    pub fn from_percentage_value(value: f64) -> Self {
        Self(value / 100.0)
    }

    pub fn from_fraction(value: f64) -> Self {
        Self(value)
    }

    pub fn as_fraction(&self) -> f64 {
        self.0
    }

    pub fn as_percentage(&self) -> f64 {
        self.0 * 100.0
    }
}

impl std::ops::Mul<f64> for Percentage {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl std::ops::Mul<f32> for Percentage {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        self.0 as f32 * rhs
    }
}

impl Default for Percentage {
    fn default() -> Self {
        Percentage(0.0)
    }
}

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Percentage::from_fraction({:.2})", self.0)
    }
}
