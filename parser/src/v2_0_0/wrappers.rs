use heck::{ToPascalCase, ToSnakeCase};

pub struct StructName(pub String);
impl StructName {
    pub fn card(s: &str) -> Self {
        Self(format!("Card{}", s.to_pascal_case()))
    }

    pub fn skill(s: &str) -> Self {
        Self(format!("Skill{}", s.to_pascal_case()))
    }

    pub fn encounter(s: &str) -> Self {
        Self(format!("Encounter{}", s.to_pascal_case()))
    }
}

pub struct ModuleName(pub String);

impl ModuleName {
    pub fn card(name: &str) -> Self {
        Self(format!("card_{}", name.to_snake_case()))
    }

    pub fn skill(name: &str) -> Self {
        Self(format!("skill_{}", name.to_snake_case()))
    }

    pub fn encounter(name: &str) -> Self {
        Self(format!("encounter_{}", name.to_snake_case()))
    }
}

pub fn tag_strlist<I: IntoIterator>(tag: &str, strlist: I) -> String
where
    <I as IntoIterator>::Item: std::fmt::Display,
{
    strlist
        .into_iter()
        .map(|v| format!("{tag}::{v}"))
        .collect::<Vec<String>>()
        .join(", ")
}
