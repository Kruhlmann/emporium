use std::path::PathBuf;

use rand::{SeedableRng, rngs::StdRng};
use simulator::{Simulation, SimulationResult, SimulationTemplate};

#[allow(unused)]
pub static SEED: u64 = 0x3a3f7af8085da7a2;

#[allow(unused)]
pub fn run_simulation(
    name: String,
    template: SimulationTemplate,
) -> Result<SimulationResult, Box<dyn std::error::Error>> {
    eprintln!("{}", name);
    let name = name.replace('"', "");
    let rng = StdRng::seed_from_u64(template.seed.unwrap_or(SEED));
    let mut simulation: Simulation = template.try_into()?;
    let result =
        tracing::info_span!("simulation", %name).in_scope(|| simulation.run_once_with_rng(rng));
    Ok(result)
}

#[allow(unused)]
pub fn read_simulation(path: &PathBuf) -> Result<SimulationTemplate, Box<dyn std::error::Error>> {
    let simulation_str = std::fs::read_to_string(path)?;
    let template = toml::from_str::<SimulationTemplate>(&simulation_str)?;
    Ok(template)
}
