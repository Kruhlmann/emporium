use std::path::PathBuf;

use ctor::ctor;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rstest::rstest;
use simulator::{Simulation, SimulationDrawType, SimulationResult, SimulationTemplate};

static SEED: u64 = 0x3a3f7af8085da7a2;

#[ctor]
fn init_tracing() {
    #[cfg(feature = "trace")]
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or("info".into()))
        .init();
}

fn read_simulation(path: &PathBuf) -> Result<SimulationTemplate, Box<dyn std::error::Error>> {
    let simulation_str = std::fs::read_to_string(path)?;
    let template = toml::from_str::<SimulationTemplate>(&simulation_str)?;
    Ok(template)
}

fn run_simulation(
    template: SimulationTemplate,
) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    let rng = StdRng::seed_from_u64(template.seed.unwrap_or(SEED));
    let mut simulation: Simulation = template.try_into()?;
    let result = simulation.run_once_with_rng(rng);
    Ok(result)
}

#[rstest]
fn test_timeout_draw(
    #[files("tests/simulations/draw/timeout/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let template = read_simulation(&path)?;
    let template_inverted = SimulationTemplate::invert(&template);

    let result = run_simulation(template)?;
    assert!(
        matches!(
            result,
            SimulationResult::Draw(SimulationDrawType::Timeout, ..)
        ),
        "Simulation `{:?}` failed: Expected `Draw(Timeout)` got `{}`",
        path.file_name().unwrap(),
        result.short_str(),
    );

    let result = run_simulation(template_inverted)?;
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
    let result = run_simulation(template)?;
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
    let template = read_simulation(&path)?;
    let result = run_simulation(template)?;
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
    let template = read_simulation(&path)?;
    let result = run_simulation(template)?;
    eprintln!("{result}");
    assert!(
        matches!(result, SimulationResult::Victory(..)),
        "Simulation `{:?}` failed: Expected `Victory` got `{}`",
        path.file_name().unwrap(),
        result.short_str()
    );
    Ok(())
}

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
    let result = run_simulation(template);
    assert!(
        matches!(result, Err(..)),
        "Simulation `{:?}` succeeded: Expected `Err` got `{result:?}`",
        path.file_name().unwrap(),
    );
    Ok(())
}
