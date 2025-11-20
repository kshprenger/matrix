use crate::{
    communication::{Event, EventId, EventType},
    simulation_handle::SIMULATION_HANDLE,
};

pub type Jiffies = usize;

/// Returns associated with this timeout EventId.
/// This will allow process to cancel it calling reset_timeout.
pub fn schedule_timeout(after: Jiffies) -> EventId {
    SIMULATION_HANDLE.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .expect("Out of simulation context")
            .submit_event_after(EventType::Timeout, after)
    })
}

pub fn reset_timeout(timeout_id: EventId) {
    SIMULATION_HANDLE.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .expect("Out of simulation context")
            .cancel_event(&Event {
                id: timeout_id,
                event_type: EventType::Timeout,
            })
    })
}
