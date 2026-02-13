use std::{cell::RefCell, cmp::Reverse, collections::BinaryHeap, rc::Rc};

use log::debug;

use crate::{
    Now, ProcessId,
    actor::{EventSubmitter, SimulationActor},
    communication::DScaleMessage,
    global,
    nursery::Nursery,
    time::Jiffies,
};

pub type TimerId = usize;

pub(crate) fn NextTimerId() -> TimerId {
    global::GlobalUniqueId()
}

pub(crate) type TimerManagerActor = Rc<RefCell<TimerManager>>;

pub(crate) struct TimerManager {
    working_timers: BinaryHeap<Reverse<(Jiffies, (ProcessId, TimerId))>>,
    nursery: Rc<Nursery>,
}

impl TimerManager {
    pub(crate) fn New(nursery: Rc<Nursery>) -> Self {
        Self {
            working_timers: BinaryHeap::new(),
            nursery,
        }
    }
}

impl SimulationActor for TimerManager {
    fn Start(&mut self) {
        // Do nothing
    }

    fn PeekClosest(&self) -> Option<Jiffies> {
        self.working_timers.peek().map(|entry| entry.0.0)
    }

    fn Step(&mut self) {
        let (_, (process_id, timer_id)) = self.working_timers.pop().expect("Should not be empty").0;
        debug!("Firing timer with TimerId {timer_id} for P{process_id}");
        self.nursery
            .Deliver(process_id, process_id, DScaleMessage::Timer(timer_id));
    }
}

impl EventSubmitter for TimerManager {
    type Event = (ProcessId, TimerId, Jiffies);

    fn Submit(&mut self, events: &mut Vec<Self::Event>) {
        events.drain(..).for_each(|(source, timer_id, after)| {
            self.working_timers
                .push(Reverse((Now() + after, (source, timer_id))));
        });
    }
}
