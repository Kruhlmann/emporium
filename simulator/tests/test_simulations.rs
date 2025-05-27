use std::path::PathBuf;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rstest::rstest;
use simulator::{Simulation, SimulationDrawType, SimulationResult, SimulationTemplate};

static SEED: u64 = 0x3a3f7af8085da7a2;

fn read_and_run_simulation(path: &PathBuf) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let simulation_str = std::fs::read_to_string(path)?;
    let template = toml::from_str::<SimulationTemplate>(&simulation_str)?;
    let rng = StdRng::seed_from_u64(template.seed.unwrap_or(SEED));
    let simulation: Simulation = template.try_into()?;
    let mut simulation = simulation.with_stdout();
    simulation.source = Some(format!("{path:?}"));
    let result = simulation.run_once_with_rng(rng);
    eprintln!("\x1b[0;31m{path:?}\x1b[0;0m");
    Ok(result)
}

#[rstest]
fn test_timeout_draw(
    #[files("tests/simulations/draw/timeout/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(&path)?;
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
    let result = read_and_run_simulation(&path)?;
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

#[rstest]
fn test_defeat(
    #[files("tests/simulations/defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(&path)?;
    assert!(
        matches!(result, SimulationResult::Defeat(..)),
        "Simulation `{:?}` failed: Expected `Defeat` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );
    Ok(())
}

#[rstest]
fn test_victory(
    #[files("tests/simulations/victory/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(&path)?;
    assert!(
        matches!(result, SimulationResult::Victory(..)),
        "Simulation `{:?}` failed: Expected `Victory` got `{}`",
        path.file_name().unwrap(),
        result.short_str()
    );
    Ok(())
}

#[rstest]
fn test_invalid(
    #[files("tests/simulations/invalid/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(&path);
    assert!(
        matches!(result, Err(..)),
        "Simulation `{:?}` succeeded: Expected `Err` got `{}`",
        path.file_name().unwrap(),
        result.unwrap().short_str(),
    );
    Ok(())
}
