use heck::ToPascalCase;
use models::v2_0_0::Tooltip;
use std::path::PathBuf;

use super::{JsonValue, ModuleName, StructName, tag_strlist};

pub struct JsonSkillFields {
    id: String,
    name: String,
    starting_tier: String,
    bronze: String,
    silver: String,
    gold: String,
    diamond: String,
    legendary: String,
    tags: String,
    hidden_tags: String,
    custom_tags: String,
    heroes: String,
    unified_tooltips: Vec<String>,
    pack_id: String,
    combat_encounters: String,
}

impl TryFrom<&serde_json::Value> for JsonSkillFields {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let JsonValue(id) = JsonValue::<String>::extract(value, &["id"])?;
        let JsonValue(name) = JsonValue::<String>::extract(value, &["name"])?;
        let JsonValue(pack_id) = JsonValue::<String>::extract(value, &["packId"])?;
        let JsonValue(starting_tier) = JsonValue::<String>::extract(value, &["startingTier"])?;
        let JsonValue(bronze) =
            JsonValue::<Vec<String>>::extract(value, &["tiers", "Bronze", "tooltips"])?;
        let JsonValue(silver) =
            JsonValue::<Vec<String>>::extract(value, &["tiers", "Silver", "tooltips"])?;
        let JsonValue(gold) =
            JsonValue::<Vec<String>>::extract(value, &["tiers", "Gold", "tooltips"])?;
        let JsonValue(diamond) =
            JsonValue::<Vec<String>>::extract(value, &["tiers", "Diamond", "tooltips"])?;
        let JsonValue(legendary) =
            JsonValue::<Vec<String>>::extract(value, &["tiers", "Legendary", "tooltips"])?;
        let JsonValue(tags_list) = JsonValue::<Vec<String>>::extract(value, &["tags"])?;
        let JsonValue(hidden_tags_list) =
            JsonValue::<Vec<String>>::extract(value, &["hiddenTags"])?;
        let JsonValue(custom_tags_list) =
            JsonValue::<Vec<String>>::extract(value, &["customTags"])?;
        let JsonValue(heroes_list) = JsonValue::<Vec<String>>::extract(value, &["heroes"])?;
        let JsonValue(unified_tooltips) =
            JsonValue::<Vec<String>>::extract(value, &["unifiedTooltips"])?;
        let JsonValue(combat_encounters_list) =
            JsonValue::<serde_json::Value>::extract(value, &["combatEncounters"])?;

        let combat_encounters = combat_encounters_list
            .as_array()
            .ok_or(anyhow::anyhow!("invalid enchantment list"))?
            .iter()
            .map(|node| {
                let JsonValue(id) = JsonValue::<String>::extract(node, &["cardId"])?;
                let JsonValue(name) = JsonValue::<String>::extract(node, &["cardName"])?;
                let s = format!(r#"CardCombatEncounter {{ id: "{id}", name: "{name}" }}"#);
                Ok(s)
            })
            .collect::<anyhow::Result<Vec<String>>>()?
            .join(",\n                ");
        let tags = tag_strlist("Tag", tags_list);
        let hidden_tags = tag_strlist("Tag", hidden_tags_list);
        let custom_tags = tag_strlist("Tag", custom_tags_list);
        let heroes = tag_strlist("Hero", heroes_list);
        Ok(Self {
            id,
            name,
            starting_tier,
            bronze: JsonSkillFields::parse_tooltip(bronze),
            silver: JsonSkillFields::parse_tooltip(silver),
            gold: JsonSkillFields::parse_tooltip(gold),
            diamond: JsonSkillFields::parse_tooltip(diamond),
            legendary: JsonSkillFields::parse_tooltip(legendary),
            tags,
            hidden_tags,
            custom_tags,
            heroes,
            unified_tooltips,
            pack_id: pack_id.to_pascal_case(),
            combat_encounters,
        })
    }
}

impl JsonSkillFields {
    fn parse_tooltip(tooltips: Vec<String>) -> String {
        tooltips
            .iter()
            .map(|t| {
                t.as_str()
                    .try_into()
                    .inspect_err(|e| eprintln!("cargo:warning={e}"))
                    .ok()
            })
            .flatten()
            .map(|s: Tooltip| s.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn to_inner_source(&self) -> String {
        format!(
            r#" Skill {{ id: "{}", name: "{}", starting_tier: Tier::{}, tiers: TieredValue {{ bronze: vec![{}], silver: vec![{}], gold: vec![{}], diamond: vec![{}], legendary: vec![{}], }}, tags: vec![{}], hidden_tags: vec![{}], custom_tags: vec![{}], heroes: vec![{}], unified_tooltips: vec!{:?}, pack_id: PackId::{}, combat_encounters: vec![{}], }} "#,
            self.id,
            self.name,
            self.starting_tier,
            self.bronze,
            self.silver,
            self.gold,
            self.diamond,
            self.legendary,
            self.tags,
            self.hidden_tags,
            self.custom_tags,
            self.heroes,
            self.unified_tooltips,
            self.pack_id,
            self.combat_encounters
        )
    }

    pub fn to_source_code(self, struct_name: &str) -> String {
        let inner = self.to_inner_source();
        format!(
            r#"// @generated
use models::v2_0_0::*; pub struct {struct_name}; impl {struct_name} {{ pub fn new() -> Skill {{ {inner} }} }} "#
        )
    }
}

pub struct SkillSourceBuilder {
    data: serde_json::Value,
}

impl SkillSourceBuilder {
    pub fn from_json_str(json: &str) -> anyhow::Result<Self> {
        let data: serde_json::Value = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    pub fn build_source_tree(&self, root: &PathBuf) -> anyhow::Result<()> {
        let skills_directory = root.join("skills");
        std::fs::create_dir_all(&skills_directory)?;

        let skills_as_json = self.data["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("no data property on root object"))?;
        let mut skills_mod_rs_source = String::from("// @generated\n");
        for json_skill in skills_as_json {
            let name = json_skill["name"]
                .as_str()
                .ok_or(anyhow::anyhow!("no name property on {json_skill:?}"))?;
            let StructName(struct_name) = StructName::skill(name);
            let ModuleName(module_name) = ModuleName::skill(name);
            let fields: JsonSkillFields = json_skill
                .try_into()
                .inspect_err(|e| eprintln!("cargo:warning=invalid json ({e}) {json_skill:?}"))?;
            let source = fields.to_source_code(&struct_name);

            let syntax_tree = syn::parse_str(&source)?;
            let formatted = prettyplease::unparse(&syntax_tree);
            std::fs::write(
                skills_directory.join(format!("{module_name}.rs")),
                formatted,
            )?;
            skills_mod_rs_source.push_str(&format!(
                "pub mod {module_name};pub use {module_name}::{struct_name};\n"
            ))
        }
        std::fs::write(skills_directory.join("mod.rs"), skills_mod_rs_source)?;
        Ok(())
    }
}
