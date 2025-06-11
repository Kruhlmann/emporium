mod aux;

use aux::{read_simulation, run_simulation};
use rstest::rstest;
use simulator::{SimulationDrawType, SimulationResult, SimulationTemplate};
use std::path::PathBuf;

#[rstest]
fn test_timeout_draw(
    #[files("tests/simulations/draw/timeout/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let template = read_simulation(&path)?;
    let template_inverted = SimulationTemplate::invert(&template);

    let result = run_simulation(format!("{:?}", path.file_name().unwrap()), template)?;
    assert!(
        matches!(
            result,
            SimulationResult::Draw(SimulationDrawType::Timeout, ..)
        ),
        "Simulation `{:?}` failed: Expected `Draw(Timeout)` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );

    let result = run_simulation(
        format!("invert_{:?}", path.file_name().unwrap()),
        template_inverted,
    )?;
    assert!(
        matches!(
            result,
            SimulationResult::Draw(SimulationDrawType::Timeout, ..)
        ),
        "Simulation `{:?}` failed: Expected `Draw(Timeout)` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );
    Ok(())
}

#[rstest]
fn test_simulations_draw(
    #[files("tests/simulations/draw/simultaneous_defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let template = read_simulation(&path)?;
    let result = run_simulation(format!("{:?}", path.file_name().unwrap()), template)?;
    assert!(
        matches!(
            result,
            SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, ..)
        ),
        "Simulation `{:?}` failed: Expected `Draw(SimultaneousDefeat)` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );
    Ok(())
}
