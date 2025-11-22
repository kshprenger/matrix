use std::ops::{Add, AddAssign};

use crate::{
    communication::{Destination, Event, EventId, EventType},
    simulation_handle::with_sim,
};

#[derive(PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub struct Jiffies(pub usize);

impl Add for Jiffies {
    type Output = Jiffies;

    fn add(self, rhs: Self) -> Self::Output {
        Jiffies(self.0 + rhs.0)
    }
}

impl AddAssign<usize> for Jiffies {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

/// Returns associated with this timeout EventId.
/// This will allow process to cancel it calling reset_timeout.
pub fn schedule_timeout(after: Jiffies) -> EventId {
    with_sim(|sim| sim.submit_event_after(EventType::Timeout, Destination::SendSelf, after))
}

pub fn reset_timeout(timeout_id: EventId) {
    with_sim(|sim| {
        sim.cancel_event(&Event {
            id: timeout_id,
            event_type: EventType::Timeout,
        })
    });
}
