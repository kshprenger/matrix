use std::cell::Cell;

use log::debug;

use crate::Jiffies;

thread_local! {
    pub(crate) static CLOCK: Cell<Jiffies> = Cell::new(Jiffies(0))
}

pub(crate) fn Drop() {
    CLOCK.take();
}

pub(crate) fn FastForwardClock(future: Jiffies) {
    let present = CLOCK.replace(future);
    debug_assert!(present <= future, "Future < Present");
    debug!("Global time now: {future}");
}

pub fn Now() -> Jiffies {
    CLOCK.get()
}
