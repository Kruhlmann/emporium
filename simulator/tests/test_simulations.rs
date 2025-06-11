use std::path::PathBuf;

use rand::rngs::StdRng;
use rand::SeedableRng;
use rstest::rstest;
use simulator::{Simulation, SimulationDrawType, SimulationResult, SimulationTemplate};
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

#[rstest]
fn test_defeat(
    #[files("tests/simulations/defeat/*.toml")] path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
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

#[rstest]
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
