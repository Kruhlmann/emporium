mod aux;

use aux::{read_simulation, run_simulation};
use rstest::rstest;
use std::path::PathBuf;

#[rstest]
fn test_invalid_template(
    #[files("tests/simulations/invalid/template/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_simulation(&path);
    assert!(
        matches!(result, Err(..)),
        "Template parsing `{:?}` succeeded: Expected `Err` got `{result:?}`",
        path.file_name().unwrap(),
    );
    Ok(())
}

#[rstest]
fn test_invalid_sim(
    #[files("tests/simulations/invalid/sim/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let template = read_simulation(&path)?;
    let result = run_simulation(format!("{:?}", path.file_name().unwrap()), template);
    assert!(
        matches!(result, Err(..)),
        "Simulation `{:?}` succeeded: Expected `Err` got `{result:?}`",
        path.file_name().unwrap(),
    );
    Ok(())
}
