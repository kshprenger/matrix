use std::{collections::HashMap, ops::Add};

use crate::process::ProcessId;

#[derive(Clone, Default)]
pub(crate) struct Metrics {
    pub events_total: usize,
    pub timeout_distribution: HashMap<ProcessId, usize>,
}

impl Metrics {
    pub(crate) fn add_timeout(&mut self, id: ProcessId) {
        self.add_event();
        *self
            .timeout_distribution
            .get_mut(&id)
            .expect(&format!("No process with id: {}", id)) += 1;
    }
    pub(crate) fn add_event(&mut self) {
        self.events_total += 1;
    }
}
