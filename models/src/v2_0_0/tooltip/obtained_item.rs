use heck::ToTitleCase;

use super::Condition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObtainedEffectItem {
    pub name: String,
    pub count: u32,
    pub condition: Option<Condition>,
}

impl ObtainedEffectItem {
    pub fn new<T: Into<String>>(name: T, count: u32) -> Self {
        Self {
            name: name.into().to_title_case(),
            count,
            condition: None,
        }
    }

    pub fn new_conditional<T: Into<String>>(name: T, count: u32, condition: Condition) -> Self {
        Self {
            name: name.into().to_title_case(),
            count,
            condition: Some(condition),
        }
    }
}

impl std::fmt::Display for ObtainedEffectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let condition_str = match &self.condition {
            Some(c) => &format!("Some({c})"),
            None => "None",
        };
        write!(
            f,
            r#"ObtainedEffectItem {{ name: "{}".to_string(), count: {}, condition: {condition_str} }}"#,
            self.name, self.count
        )
    }
}
