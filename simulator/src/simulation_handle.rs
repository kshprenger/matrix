use std::cell::RefCell;

use crate::simulation::Simulation;

thread_local! {
    pub static SIMULATION_HANDLE: RefCell<Option<Simulation>> = RefCell::new(None);
}

pub(crate) fn setup_ctx(sim: Simulation) {
    SIMULATION_HANDLE.set(Some(sim));
}
