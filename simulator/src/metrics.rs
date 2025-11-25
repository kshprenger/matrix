use crate::history::ProcessStep;

#[derive(Clone, Default)]
pub struct Metrics {
    pub events_total: usize,
}

impl Metrics {
    pub(crate) fn track_event(&mut self) {
        self.events_total += 1;
        if self.events_total % 1_000_000 == 0 {
            println!("Progress: {}", self.events_total)
        }
    }
}
