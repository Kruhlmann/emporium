#[derive(Copy, Clone, Debug)]
pub struct PlayerHealth(pub i64, pub u64);

impl PlayerHealth {
    pub fn max(&self) -> u64 {
        self.1
    }

    pub fn current(&self) -> i64 {
        self.0
    }

    pub fn fraction(&self) -> f32 {
        (self.0 as f64 / self.1 as f64) as f32
    }
}

impl std::fmt::Display for PlayerHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Health ({}/{})", self.current(), self.max())
    }
}

impl std::ops::Add<i64> for PlayerHealth {
    type Output = Self;

    fn add(self, other: i64) -> Self::Output {
        Self(std::cmp::min(self.1 as i64, self.0 + other), self.1)
    }
}

impl std::ops::Sub<i64> for PlayerHealth {
    type Output = Self;

    fn sub(self, other: i64) -> Self::Output {
        Self(std::cmp::min(self.1 as i64, self.0 - other), self.1)
    }
}

impl std::ops::AddAssign<i64> for PlayerHealth {
    fn add_assign(&mut self, other: i64) {
        self.0 = std::cmp::min(self.1 as i64, self.0 + other);
    }
}

impl std::ops::SubAssign<i64> for PlayerHealth {
    fn sub_assign(&mut self, other: i64) {
        self.0 = std::cmp::min(self.1 as i64, self.0 - other);
    }
}
