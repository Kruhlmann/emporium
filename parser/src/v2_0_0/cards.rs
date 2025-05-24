use heck::ToPascalCase;
use models::v2_0_0::Tooltip;
use std::path::PathBuf;

use crate::HTTP_REQUEST_THROTTLE;

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

    #[cfg(feature = "thumbnail-backdrops")]
    async fn write_thumbnail_backdrop_files(
        image_path: &PathBuf,
        image_backdrop_path: &PathBuf,
        size: &str,
    ) -> anyhow::Result<()> {
        use image::{DynamicImage, GenericImageView, ImageFormat};
        let image_path = image_path.clone();
        let image_backdrop_path = image_backdrop_path.clone();
        let size = size.to_string();
        tokio::task::spawn_blocking(move || {
            if std::fs::exists(&image_backdrop_path)? {
                return Ok::<(), anyhow::Error>(());
            }
            eprintln!("cargo:warning=backdrop for {image_backdrop_path:?} not found; generating",);
            let img = image::open(&image_path)?;
            let (w, h) = img.dimensions();

            let mut combined = image_path.clone();
            combined.set_file_name(format!(
                "{}.backdrop.{}",
                image_path.file_stem().unwrap().to_string_lossy(),
                image_path.extension().unwrap().to_string_lossy(),
            ));

            if size == "Large" {
                std::fs::copy(&image_path, &combined)?;
                return Ok(());
            }

            let large_w = 384; // Medium item image is 256px
            let large_h = ((large_w as f32) * (h as f32) / (w as f32)) as u32;
            let resized = image::imageops::resize(
                &img,
                large_w,
                large_h,
                image::imageops::FilterType::Gaussian,
            );

            let blurred = image::imageops::blur(&resized, 30.0);

            let span = match size.as_str() {
                "Small" => 1,
                "Medium" => 2,
                s => panic!("invalid size {s}"),
            };
            let slot_w_px = large_w / 3;
            let orig_w = slot_w_px * span;
            let orig_h = ((orig_w as f32) * (h as f32) / (w as f32)) as u32;
            let orig_scaled = image::imageops::resize(
                &img,
                orig_w,
                orig_h,
                image::imageops::FilterType::Lanczos3,
            );

            let mut canvas = DynamicImage::ImageRgba8(blurred).to_rgba8();
            let x = (large_w - orig_w) / 2;
            let y = (large_h - orig_h) / 2;
            image::imageops::overlay(&mut canvas, &orig_scaled, x.into(), y.into());

            DynamicImage::ImageRgba8(canvas).save_with_format(&combined, ImageFormat::Avif)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    #[cfg(feature = "thumbnails")]
    async fn write_thumbnail_files(
        image_id: &str,
        image_directory: &PathBuf,
        size: &str,
    ) -> anyhow::Result<()> {
        eprintln!(
            "cargo:warning=building thumbnails from scratch. this creates a lot of network traffic and may not be appreciated by the asset hosts. use sparingly"
        );
        let image_filename = format!("{}.avif", image_id);
        let image_path = image_directory.join(&image_filename);
        let image_backdrop_filename = format!("{}.backdrop.avif", image_id);
        let image_backdrop_path = image_directory.join(&image_backdrop_filename);

        if let Ok(true) = std::fs::exists(&image_path) {
            if cfg!(feature = "thumbnail-backdrops") {
                return CardSourceBuilder::write_thumbnail_backdrop_files(
                    &image_path,
                    &image_backdrop_path,
                    size,
                )
                .await;
            } else {
                return Ok(());
            }
        }

        let _permit = HTTP_REQUEST_THROTTLE.acquire().await;

        let mut file = std::fs::File::create(&image_path)?;
        let url = format!(
            "https://howbazaar-images.b-cdn.net/images/items/{}",
            image_filename
        );
        eprintln!("cargo:warning=image not found; downloading from {url}");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let response = reqwest::get(&url).await?;
        response.error_for_status_ref()?;
        let mut content = response.bytes().await?;
        std::io::copy(&mut content.as_ref(), &mut file).inspect_err(|error| {
            eprintln!(
                "cargo:error=unable to set file content {:?}: {error}",
                image_path
            )
        })?;

        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        if file_size == 0 {
            eprintln!("cargo:warning=empty file created from {url} at {image_filename}");
            std::fs::remove_file(&image_path)?;
        }

        #[cfg(feature = "thumbnail-backdrops")]
        std::fs::remove_file(&image_backdrop_path)?;
        if cfg!(feature = "thumbnail-backdrops") {
            CardSourceBuilder::write_thumbnail_backdrop_files(
                &image_path,
                &image_backdrop_path,
                size,
            )
            .await
        } else {
            Ok(())
        }
    }

    pub async fn build_source_tree(self, root: &PathBuf) -> anyhow::Result<()> {
        let card_directory = root.join("cards");
        let image_directory = card_directory.join("images");

        tokio::fs::create_dir_all(&image_directory).await?;

        let cards = self.data["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("no data array"))?
            .clone();

        let mut handles = Vec::with_capacity(cards.len());

        for json_card in cards {
            let card_directory = card_directory.clone();
            #[cfg(feature = "thumbnails")]
            let image_directory = image_directory.clone();
            handles.push(tokio::spawn(async move {
                let name = json_card["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("card missing name"))?;
                let name_owned = name.to_string();
                let fields: JsonCardFields = (&json_card)
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("invalid card JSON: {e}"))?;

                #[cfg(feature = "thumbnails")]
                CardSourceBuilder::write_thumbnail_files(
                    &fields.id,
                    &image_directory,
                    &fields.size,
                )
                .await?;

                let name_returned = name_owned.clone();
                let (module_name, struct_name, source_code) =
                    tokio::task::spawn_blocking(move || {
                        let StructName(struct_name) = StructName::card(&name_owned);
                        let ModuleName(module_name) = ModuleName::card(&name_owned);
                        let source = fields.to_source_code(&struct_name);
                        let syntax_tree = syn::parse_str(&source).inspect_err(|e| {
                            eprintln!("cargo:warning=invalid rust syntax ({e}):\n{source}");
                        })?;
                        let formatted = prettyplease::unparse(&syntax_tree);
                        Ok::<_, anyhow::Error>((module_name, struct_name, formatted))
                    })
                    .await??;

                let file_path = card_directory.join(format!("{}.rs", module_name));
                tokio::fs::write(&file_path, source_code).await?;

                Ok::<(String, String, String), anyhow::Error>((
                    module_name,
                    struct_name,
                    name_returned,
                ))
            }));
        }

        let mut struct_metadata_list = Vec::new();
        for handle in handles {
            struct_metadata_list.push(handle.await??);
        }

        // Generate mod.rs
        let mut mod_src = String::from("// @generated\n");
        for (module_name, struct_name, _) in &struct_metadata_list {
            mod_src.push_str(&format!(
                "pub mod {module_name}; pub use {module_name}::{struct_name};\n",
            ));
        }
        mod_src.push_str("lazy_static::lazy_static!{\n    pub static ref CONSTRUCT_CARD_BY_NAME: std::collections::HashMap<&'static str, fn() -> models::v2_0_0::Card> = std::collections::HashMap::from([\n");
        for (module_name, struct_name, name) in &struct_metadata_list {
            mod_src.push_str(&format!(
                r#"        ("{name}", {module_name}::{struct_name}::new as fn() -> _),"#
            ));
            mod_src.push_str("\n");
        }
        mod_src.push_str("\n    ]);\n}\n");
        tokio::fs::write(card_directory.join("mod.rs"), mod_src).await?;
        Ok(())
    }
}
