use std::time::Duration;

use crate::TICK_DURATION;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct GameTicks(pub u128);

impl From<Duration> for GameTicks {
    fn from(value: Duration) -> Self {
        Self(value.as_millis() / TICK_DURATION.as_millis())
    }
}

impl std::fmt::Display for GameTicks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_duration = Duration::from_millis((TICK_DURATION.as_millis() * self.0) as u64);
        write!(f, "GameTicks<{}> ({:#?})", self.0, as_duration)
    }
}

impl Default for GameTicks {
    fn default() -> Self {
        Self(0)
    }
}

impl std::ops::Add for GameTicks {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GameTicks(self.0 + other.0)
    }
}

impl std::ops::Sub for GameTicks {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        GameTicks(self.0 - other.0)
    }
}

impl std::ops::Mul for GameTicks {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        GameTicks(self.0 * other.0)
    }
}

impl std::ops::Div for GameTicks {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        GameTicks(self.0 / other.0)
    }
}

impl std::ops::AddAssign for GameTicks {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for GameTicks {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl std::ops::MulAssign for GameTicks {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}

impl std::ops::DivAssign for GameTicks {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0;
    }
}

impl std::ops::Add<u128> for GameTicks {
    type Output = Self;
    fn add(self, rhs: u128) -> Self {
        GameTicks(self.0 + rhs)
    }
}

impl std::ops::Sub<u128> for GameTicks {
    type Output = Self;

    fn sub(self, rhs: u128) -> Self {
        GameTicks(self.0 - rhs)
    }
}

impl std::ops::Mul<u128> for GameTicks {
    type Output = Self;

    fn mul(self, rhs: u128) -> Self {
        GameTicks(self.0 * rhs)
    }
}

impl std::ops::Div<u128> for GameTicks {
    type Output = Self;

    fn div(self, rhs: u128) -> Self {
        GameTicks(self.0 / rhs)
    }
}

impl std::ops::AddAssign<u128> for GameTicks {
    fn add_assign(&mut self, rhs: u128) {
        self.0 += rhs;
    }
}

impl std::ops::SubAssign<u128> for GameTicks {
    fn sub_assign(&mut self, rhs: u128) {
        self.0 -= rhs;
    }
}

impl std::ops::MulAssign<u128> for GameTicks {
    fn mul_assign(&mut self, rhs: u128) {
        self.0 *= rhs;
    }
}

impl std::ops::DivAssign<u128> for GameTicks {
    fn div_assign(&mut self, rhs: u128) {
        self.0 /= rhs;
    }
}
