use std::path::PathBuf;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rstest::rstest;
use simulator::simulation::Simulation;
use simulator::simulation::SimulationDrawType;
use simulator::simulation::SimulationResult;
use simulator::simulation::SimulationTemplate;

static SEED: u64 = 0x3a3f7af8085da7a2;

fn read_and_run_simulation(path: PathBuf) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let rng = StdRng::seed_from_u64(SEED);
    let simulation_str = std::fs::read_to_string(&path)?;
    let mut simulation: Simulation =
        toml::from_str::<SimulationTemplate>(&simulation_str)?.try_into()?;
    simulation.source = Some(format!("{path:?}"));
    let result = simulation.run_once_with_rng(rng);
    eprintln!("\x1b[0;31m{path:?}\x1b[0;0m");
    Ok(result)
}

#[rstest]
fn test_timeout_draw(
    #[files("tests/simulations/draw/timeout/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(path)?;
    assert!(matches!(
        result,
        SimulationResult::Draw(SimulationDrawType::Timeout, ..)
    ));
    Ok(())
}

#[rstest]
fn test_simulations_draw(
    #[files("tests/simulations/draw/simultaneous_defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(path)?;
    assert!(matches!(
        result,
        SimulationResult::Draw(SimulationDrawType::SimultaneousDefeat, ..)
    ));
    Ok(())
}

#[rstest]
fn test_defeat(
    #[files("tests/simulations/defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(path)?;
    assert!(matches!(result, SimulationResult::Defeat(..)));
    Ok(())
}

#[rstest]
fn test_victory(
    #[files("tests/simulations/victory/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(path)?;
    assert!(matches!(result, SimulationResult::Victory(..)));
    Ok(())
}

#[rstest]
fn test_invalid(
    #[files("tests/simulations/invalid/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = read_and_run_simulation(path);
    assert!(matches!(result, Err(..)));
    Ok(())
}
