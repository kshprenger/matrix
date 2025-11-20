use crate::simulation_handle::SIMULATION_HANDLE;

pub type Jiffies = usize;

pub fn schedule_timeout(after: Jiffies) {
    SIMULATION_HANDLE.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .map(|sim| sim.submit_event_after(crate::communication::EventType::Timeout, after))
            .or_else(|| panic!("Out of simulation context"))
    });
}

/// We assume that process schedules at most one global timeout.
/// Maybe this behaviour will be generialized.
pub fn reset_timeout() {
    todo!()
}
