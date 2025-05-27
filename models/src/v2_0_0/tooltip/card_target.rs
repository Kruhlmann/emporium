use super::TargetCondition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardTarget(pub usize, pub TargetCondition);

impl CardTarget {
    pub fn number_of_targets(&self) -> usize {
        self.0
    }

    pub fn target_condition(&self) -> &TargetCondition {
        &self.1
    }
}

impl std::fmt::Display for CardTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CardTarget({}, {})", self.0, self.1)
    }
}
