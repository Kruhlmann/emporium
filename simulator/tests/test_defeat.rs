mod aux;

use aux::{read_simulation, run_simulation};
use rstest::rstest;
use simulator::SimulationResult;
use std::path::PathBuf;

#[rstest]
fn test_defeat(
    #[files("tests/simulations/defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use simulator::SimulationTemplate;

    let template = read_simulation(&path)?;
    let template_inverted = SimulationTemplate::invert(&template);
    let result = run_simulation(format!("{:?}", path.file_name().unwrap()), template)?;
    assert!(
        matches!(result, SimulationResult::Defeat(..)),
        "Simulation `{:?}` failed: Expected `Defeat` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );
    let result = run_simulation(format!("invert_{:?}", path.file_name()), template_inverted)?;
    assert!(
        matches!(result, SimulationResult::Victory(..)),
        "Simulation `{:?}` failed: Expected `Draw(Timeout)` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );
    Ok(())
}
