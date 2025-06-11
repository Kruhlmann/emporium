use simulator::{DispatchableEvent, Simulation, SimulationResult};

pub fn spawn_run_simulation_thread(
    chunk: usize,
    simulation: Simulation,
    thread_res_tx: std::sync::mpsc::Sender<SimulationResult>,
    thread_evt_tx: std::sync::mpsc::Sender<DispatchableEvent>,
) {
    std::thread::spawn(move || {
        let base_rng = Simulation::create_rng();
        let rng_clone = base_rng.clone();
        for _ in 0..chunk {
            let mut sim = simulation.clone().with_channel(thread_evt_tx.clone());
            let result = sim.run_once_with_rng(rng_clone.clone());
            if let Err(error) = thread_res_tx.send(result) {
                tracing::error!(?error, "error running simulation");
                break;
            }
        }
    });
}
