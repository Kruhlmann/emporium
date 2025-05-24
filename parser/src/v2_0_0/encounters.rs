use std::path::PathBuf;

use super::{JsonCardFields, JsonSkillFields, JsonValue, ModuleName, StructName};

pub struct JsonEncounterSkillFields {
    skill: JsonSkillFields,
    tier: String,
}

impl TryFrom<&serde_json::Value> for JsonEncounterSkillFields {
    type Error = anyhow::Error;

    fn try_from(input: &serde_json::Value) -> Result<Self, Self::Error> {
        let skill_reference = &input["card"];
        let skill: JsonSkillFields = skill_reference.try_into()?;
        let tier = input["tierType"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(anyhow::anyhow!("missing tier type on encounter skill"))?;

        Ok(Self { skill, tier })
    }
}

impl JsonEncounterSkillFields {
    pub fn to_inner_source(&self) -> String {
        let skill_src = self.skill.to_inner_source();
        format!(
            r#"
            TieredSkill {{
                tier: Tier::{},
                skill: {skill_src},
            }}
        "#,
            self.tier,
        )
    }
}

pub struct JsonEncounterCardFields {
    card: JsonCardFields,
    tier: String,
    enchantment: Option<String>,
}

impl TryFrom<&serde_json::Value> for JsonEncounterCardFields {
    type Error = anyhow::Error;

    fn try_from(input: &serde_json::Value) -> Result<Self, Self::Error> {
        let card_reference = &input["card"];
        let card: JsonCardFields = card_reference.try_into()?;
        let enchantment = input["enchantmentType"].as_str().map(|s| s.to_string());
        let tier = input["tierType"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(anyhow::anyhow!("missing tier type on encounter card"))?;

        Ok(Self {
            card,
            enchantment,
            tier,
        })
    }
}

impl JsonEncounterCardFields {
    pub fn to_inner_source(&self) -> String {
        let card_src = self.card.to_inner_source();
        let enchantment = match &self.enchantment {
            Some(e) => &format!("Some(Enchantment::{e})"),
            None => "None",
        };
        format!(
            r#"EncounterCard {{ enchantment: {enchantment}, card: TieredCard {{ tier: Tier::{}, card: {card_src}, }}, }}"#,
            self.tier
        )
    }
}

pub struct JsonEncounterFields {
    id: String,
    name: String,
    level: i64,
    health: i64,
    day: String,
    cards: String,
    skills: String,
}

impl TryFrom<&serde_json::Value> for JsonEncounterFields {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let JsonValue(id) = JsonValue::<String>::extract(value, &["cardId"])?;
        let JsonValue(name) = JsonValue::<String>::extract(value, &["cardName"])?;
        let JsonValue(level) = JsonValue::<i64>::extract(value, &["level"])?;
        let JsonValue(health) = JsonValue::<i64>::extract(value, &["health"])?;
        let JsonValue(cards) = JsonValue::<serde_json::Value>::extract(value, &["items"])?;
        let JsonValue(skills) = JsonValue::<serde_json::Value>::extract(value, &["skills"])?;

        let cards = cards
            .as_array()
            .ok_or(anyhow::anyhow!("invalid encounter cards json"))?
            .iter()
            .map(|c| c.try_into())
            .collect::<anyhow::Result<Vec<JsonEncounterCardFields>>>()?
            .iter()
            .map(|c| c.to_inner_source())
            .collect::<Vec<String>>()
            .join(",");
        let skills = skills
            .as_array()
            .ok_or(anyhow::anyhow!("invalid encounter skills json"))?
            .iter()
            .map(|c| c.try_into())
            .collect::<anyhow::Result<Vec<JsonEncounterSkillFields>>>()?
            .iter()
            .map(|c| c.to_inner_source())
            .collect::<Vec<String>>()
            .join(",");
        let day = value["day"]
            .as_i64()
            .map(|i| format!("Numeric({i})"))
            .unwrap_or("Event".to_string());
        Ok(Self {
            id,
            name,
            level,
            health,
            day,
            cards,
            skills,
        })
    }
}

impl JsonEncounterFields {
    pub fn to_source_code(self, struct_name: &str) -> String {
        format!(
            r#"// @generated
use models::v2_0_0::*; pub struct {struct_name}; impl {struct_name} {{ pub fn new() -> Encounter {{ Encounter {{ id: "{}", name: "{}", level: {}, health: {}, day: EncounterDay::{}, cards: vec![{}], skills: vec![{}], }} }} }} "#,
            self.id, self.name, self.level, self.health, self.day, self.cards, self.skills
        )
    }
}

pub struct EncounterSourceBuilder {
    data: serde_json::Value,
}

impl EncounterSourceBuilder {
    pub fn from_json_str(json: &str) -> anyhow::Result<Self> {
        let data: serde_json::Value = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    pub fn build_source_tree(&self, root: &PathBuf) -> anyhow::Result<()> {
        let encounter_directory = root.join("encounters");
        std::fs::create_dir_all(&encounter_directory)?;

        let mut encounters_mod_rs_source = String::from("// @generated\n");
        let encounter_days_json = self.data["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("no data property on root object"))?;

        for day_json in encounter_days_json {
            let day_module_name = day_json["day"]
                .as_i64()
                .map(|s| format!("day_{s}"))
                .unwrap_or("day_event".to_string());
            encounters_mod_rs_source.push_str(&format!("pub mod {day_module_name};\n"));
            let day_directory = &encounter_directory.join(day_module_name);
            std::fs::create_dir_all(day_directory)?;
            let mut day_mod_rs = String::from("// @generated\n");
            let groups = day_json["groups"]
                .as_array()
                .ok_or(anyhow::anyhow!("groups cannot be parsed {day_json:?}"))?;
            for group in groups {
                let encounters = group
                    .as_array()
                    .ok_or(anyhow::anyhow!("groups cannot be parsed {day_json:?}"))?;
                for json_encounter in encounters {
                    let name = json_encounter["cardName"].as_str().ok_or(anyhow::anyhow!(
                        "no cardName property on {json_encounter:?}"
                    ))?;
                    let StructName(struct_name) = StructName::encounter(name);
                    let ModuleName(module_name) = ModuleName::encounter(name);
                    let fields: JsonEncounterFields = json_encounter.try_into()?;
                    let source = fields.to_source_code(&struct_name);

                    let syntax_tree = syn::parse_str(&source)?;
                    let formatted = prettyplease::unparse(&syntax_tree);
                    std::fs::write(day_directory.join(format!("{module_name}.rs")), formatted)?;
                    day_mod_rs.push_str(&format!(
                        "pub mod {module_name};\npub use {module_name}::{struct_name};\n"
                    ));
                }
            }
            std::fs::write(day_directory.join("mod.rs"), day_mod_rs)?;
        }
        std::fs::write(encounter_directory.join("mod.rs"), encounters_mod_rs_source)?;
        Ok(())
    }
}
