use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use log::debug;

use crate::{
    ProcessId, access,
    actor::SimulationActor,
    process::SharedProcessHandle,
    time::{Jiffies, Now},
    tso::NextGlobalUniqueId,
};

pub type TimerId = usize;

pub(crate) fn NextTimerId() -> TimerId {
    NextGlobalUniqueId()
}

// We cannot cancel timers yet. So user tracks them using TimerId
pub(crate) struct Timers {
    working_timers: BinaryHeap<Reverse<(Jiffies, (ProcessId, TimerId))>>,
    procs: HashMap<ProcessId, SharedProcessHandle>,
}

impl Timers {
    pub(crate) fn New(procs: HashMap<ProcessId, SharedProcessHandle>) -> Self {
        Self {
            working_timers: BinaryHeap::new(),
            procs,
        }
    }

    pub(crate) fn ScheduleTimers(&mut self, timers: &mut Vec<(ProcessId, TimerId, Jiffies)>) {
        timers
            .drain(..)
            .into_iter()
            .for_each(|(source, timer_id, after)| {
                debug!("submitted timer to fire at {}", Now() + after);
                self.working_timers
                    .push(Reverse((Now() + after, (source, timer_id))));
            });
    }
}

impl SimulationActor for Timers {
    fn Start(&mut self) {
        // Do nothing
    }

    fn PeekClosest(&self) -> Option<Jiffies> {
        self.working_timers.peek().map(|entry| entry.0.0)
    }

    fn Step(&mut self) {
        let (_, (process_id, timer_id)) = self.working_timers.pop().expect("Should not be empty").0;
        access::SetProcess(process_id);
        debug!("Firing timer with TimerId {timer_id} for Process {process_id}");
        self.procs
            .get_mut(&process_id)
            .expect("Invalid ProcessId")
            .borrow_mut()
            .OnTimer(timer_id);
    }
}
