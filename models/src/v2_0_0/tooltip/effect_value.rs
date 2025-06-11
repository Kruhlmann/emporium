#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectValue<T> {
    Flat(T),
    Percentage(T),
}
static TODO: bool = true; //TODO Get rid of this, we have Percentage(..)

impl<T> std::fmt::Display for EffectValue<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectValue::Flat(i) => write!(f, "EffectValue::Flat({i})"),
            EffectValue::Percentage(i) => write!(f, "EffectValue::Percentage({i})"),
        }
    }
}
