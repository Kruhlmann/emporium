mod aux;

use aux::{read_simulation, run_simulation};
use simulator::SimulationResult;
use std::path::PathBuf;

#[rstest::rstest]
fn test_victory(
    #[files("tests/simulations/victory/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let template = read_simulation(&path)?;
    let result = run_simulation(format!("{:?}", path.file_name().unwrap()), template)?;
    assert!(
        matches!(result, SimulationResult::Victory(..)),
        "Simulation `{:?}` failed: Expected `Victory` got `{}`",
        path.file_name().unwrap(),
        result.short_str()
    );
    Ok(())
}
