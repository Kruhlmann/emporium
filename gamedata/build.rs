use parser::v2_0_0::{CardSourceBuilder, EncounterSourceBuilder, SkillSourceBuilder};
use std::{path::PathBuf, sync::Arc};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=res/2.0.0/");

    let card_root = Arc::new(PathBuf::from("src/v2_0_0"));
    let skill_root = Arc::clone(&card_root);
    let encounter_root = Arc::clone(&card_root);
    let mut lib_rs_source = String::new();

    for version in [(2, 0, 0)] {
        let v_str_underscore = format!("{}_{}_{}", version.0, version.1, version.2);
        let v_str_dot = format!("{}.{}.{}", version.0, version.1, version.2);
        lib_rs_source.push_str(&format!("pub mod v{v_str_underscore};\n"));

        let card_root = card_root.clone();
        let skill_root = skill_root.clone();
        let encounter_root = encounter_root.clone();

        let card_json_file_path = format!("res/{v_str_dot}/cards.json");
        let skill_json_file_path = format!("res/{v_str_dot}/skills.json");
        let encounter_json_file_path = format!("res/{v_str_dot}/encounters.json");

        let card_pipeline = async move {
            let json = tokio::fs::read_to_string(card_json_file_path).await?;
            let builder = CardSourceBuilder::from_json_str(&json)?;
            builder.build_source_tree(&card_root).await
        };

        let skill_pipeline = async move {
            let json = tokio::fs::read_to_string(skill_json_file_path).await?;
            let builder = SkillSourceBuilder::from_json_str(&json)?;
            builder.build_source_tree(&skill_root)
        };

        let encounter_pipeline = async move {
            let json = tokio::fs::read_to_string(encounter_json_file_path).await?;
            let builder = EncounterSourceBuilder::from_json_str(&json)?;
            builder.build_source_tree(&encounter_root)
        };

        let mut joinset = JoinSet::new();
        joinset.spawn(card_pipeline);
        joinset.spawn(skill_pipeline);
        joinset.spawn(encounter_pipeline);

        while let Some(res) = joinset.join_next().await {
            res??;
        }
    }

    tokio::fs::write("src/lib.rs", format!("// @generated\n{lib_rs_source}")).await?;
    Ok(())
}
