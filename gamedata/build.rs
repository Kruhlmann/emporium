fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=res/2.0.0/");
    let root = std::path::Path::new("src/v2_0_0").to_path_buf();
    let mut lib_rs_source = String::new();
    lib_rs_source += "pub mod v2_0_0;\n";
    let card_json = std::fs::read_to_string("res/2.0.0/cards.json")?;
    let card_source_builder = parser::v2_0_0::CardSourceBuilder::from_json_str(&card_json)?;
    card_source_builder.build_source_tree(root.clone())?;

    let skill_json = std::fs::read_to_string("res/2.0.0/skills.json")?;
    let skill_source_builder = parser::v2_0_0::SkillSourceBuilder::from_json_str(&skill_json)?;
    skill_source_builder.build_source_tree(root.clone())?;

    let encounter_json = std::fs::read_to_string("res/2.0.0/encounters.json")?;
    let encounter_source_builder =
        parser::v2_0_0::EncounterSourceBuilder::from_json_str(&encounter_json)?;
    encounter_source_builder.build_source_tree(root.clone())?;

    std::fs::write("src/lib.rs", format!("// @generated\n{lib_rs_source}"))?;
    Ok(())
}
