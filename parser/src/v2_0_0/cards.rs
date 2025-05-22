use heck::ToPascalCase;
use models::v2_0_0::Tooltip;
use std::path::PathBuf;

use super::{JsonValue, ModuleName, StructName, tag_strlist};

pub struct JsonCardFields {
    id: String,
    name: String,
    starting_tier: String,
    size: String,
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
    enchantments: String,
}

impl TryFrom<&serde_json::Value> for JsonCardFields {
    type Error = anyhow::Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let JsonValue(id) = JsonValue::<String>::extract(value, &["id"])?;
        let JsonValue(name) = JsonValue::<String>::extract(value, &["name"])?;
        let JsonValue(pack_id) = JsonValue::<String>::extract(value, &["packId"])?;
        let JsonValue(starting_tier) = JsonValue::<String>::extract(value, &["startingTier"])?;
        let JsonValue(size) = JsonValue::<String>::extract(value, &["size"])?;
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
        let JsonValue(enchantment_list) =
            JsonValue::<serde_json::Value>::extract(value, &["enchantments"])?;

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
        let enchantments = enchantment_list // TODO: don't have it on cooldown - need custom
            // enchant parser
            .as_array()
            .ok_or(anyhow::anyhow!("invalid enchantment list"))?
            .iter()
            .map(|node| {
                let JsonValue(ty) = JsonValue::<String>::extract(node, &["type"])?;
                let JsonValue(tooltips_str) =
                    JsonValue::<Vec<String>>::extract(node, &["tooltips"])?;
                let tooltips = tooltips_str
                    .iter()
                    .map(|t| Tooltip::from_or_raw(t.as_str()))
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                Ok(format!("CardEnchantment::{ty}(vec![{tooltips}])"))
            })
            .collect::<anyhow::Result<Vec<String>>>()?
            .join(", ");
        let tags = tag_strlist("Tag", tags_list);
        let hidden_tags = tag_strlist("Tag", hidden_tags_list);
        let custom_tags = tag_strlist("Tag", custom_tags_list);
        let heroes = tag_strlist("Hero", heroes_list);
        Ok(Self {
            id,
            name,
            starting_tier,
            size,
            bronze: JsonCardFields::parse_tooltip(bronze),
            silver: JsonCardFields::parse_tooltip(silver),
            gold: JsonCardFields::parse_tooltip(gold),
            diamond: JsonCardFields::parse_tooltip(diamond),
            legendary: JsonCardFields::parse_tooltip(legendary),
            tags,
            hidden_tags,
            custom_tags,
            heroes,
            unified_tooltips,
            pack_id: pack_id.to_pascal_case(),
            combat_encounters,
            enchantments,
        })
    }
}

impl JsonCardFields {
    fn parse_tooltip(tooltips: Vec<String>) -> String {
        tooltips
            .iter()
            .map(|t| {
                t.as_str()
                    .try_into()
                    .inspect_err(|e| println!("cargo:warning={e}"))
                    .ok()
            })
            .flatten()
            .map(|s: Tooltip| s.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn to_inner_source(&self) -> String {
        format!(
            r#"Card {{ id: "{}", name: "{}", starting_tier: Tier::{}, size: Size::{}, tiers: TieredValue {{ bronze: vec![{}], silver: vec![{}], gold: vec![{}], diamond: vec![{}], legendary: vec![{}], }}, tags: vec![{}], hidden_tags: vec![{}], custom_tags: vec![{}], heroes: vec![{}], enchantments: vec![ {} ], unified_tooltips: vec!{:?}, pack_id: PackId::{}, combat_encounters: vec![{}], }}"#,
            self.id,
            self.name,
            self.starting_tier,
            self.size,
            self.bronze,
            self.silver,
            self.gold,
            self.diamond,
            self.legendary,
            self.tags,
            self.hidden_tags,
            self.custom_tags,
            self.heroes,
            self.enchantments,
            self.unified_tooltips,
            self.pack_id,
            self.combat_encounters
        )
    }

    pub fn to_source_code(self, struct_name: &str) -> String {
        format!(
            r#"// @generated\n
use models::v2_0_0::*; pub struct {struct_name}; impl {struct_name} {{ pub fn new() -> Card {{ {} }} }}"#,
            self.to_inner_source(),
        )
    }
}

pub struct CardSourceBuilder {
    data: serde_json::Value,
}

impl CardSourceBuilder {
    pub fn from_json_str(json: &str) -> anyhow::Result<Self> {
        let data: serde_json::Value = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    pub fn build_source_tree(&self, root: PathBuf) -> anyhow::Result<()> {
        let cards_directory = root.join("cards");
        std::fs::create_dir_all(&cards_directory).inspect_err(|e| {
            println!("cargo:warning=unable to create directory {cards_directory:?} ({e})")
        })?;

        let cards_as_json = self.data["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("no data property on root object"))?;
        let mut struct_metadata: Vec<(String, String, String)> = Vec::new();
        let mut cards_mod_rs_source = String::from("// @generated\n");
        for json_card in cards_as_json {
            let name = json_card["name"]
                .as_str()
                .ok_or(anyhow::anyhow!("no name property for"))?;
            println!("cargo:warning={name}");
            let StructName(struct_name) = StructName::card(name);
            let ModuleName(module_name) = ModuleName::card(name);
            struct_metadata.push((name.to_string(), module_name.clone(), struct_name.clone()));
            let fields: JsonCardFields = json_card
                .try_into()
                .inspect_err(|e| println!("cargo:warning=invalid json ({e}) {json_card:?}"))?;
            let source = fields.to_source_code(&struct_name);

            let syntax_tree = syn::parse_str(&source).inspect_err(|e| {
                println!("cargo:warning=invalid rust syntax ({e}):\n{source}");
            })?;
            let formatted = prettyplease::unparse(&syntax_tree);
            std::fs::write(cards_directory.join(format!("{module_name}.rs")), formatted)?;
            cards_mod_rs_source.push_str(&format!(
                "pub mod {module_name};\npub use {module_name}::{struct_name};\n"
            ));
        }
        cards_mod_rs_source.push_str("lazy_static::lazy_static!{\n    pub static ref CONSTRUCT_CARD_BY_NAME: std::collections::HashMap<&'static str, fn() -> models::v2_0_0::Card> = std::collections::HashMap::from([\n");
        for (name, module_name, struct_name) in struct_metadata {
            cards_mod_rs_source.push_str(&format!(
                r#"        ("{name}", {module_name}::{struct_name}::new as fn() -> _),"#
            ));
            cards_mod_rs_source.push_str("\n");
        }
        cards_mod_rs_source.push_str("\n    ]);\n}\n");
        std::fs::write(cards_directory.join("mod.rs"), cards_mod_rs_source)?;
        Ok(())
    }
}
