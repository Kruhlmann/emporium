mod aux;

use aux::{read_simulation, run_simulation};
use rstest::rstest;
use simulator::SimulationResult;
use std::path::PathBuf;

#[rstest]
fn test_defeat(
    #[files("tests/simulations/defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let template = read_simulation(&path)?;
    let result = run_simulation(format!("{:?}", path.file_name().unwrap()), template)?;
    assert!(
        matches!(result, SimulationResult::Defeat(..)),
        "Simulation `{:?}` failed: Expected `Defeat` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );
    Ok(())
}
