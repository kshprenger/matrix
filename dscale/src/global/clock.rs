use std::cell::Cell;

use log::debug;

use crate::Jiffies;

thread_local! {
    pub(crate) static CLOCK: Cell<Jiffies> = Cell::new(Jiffies(0))
}

pub(crate) fn drop_clock() {
    CLOCK.take();
}

pub(crate) fn fast_forward_clock(future: Jiffies) {
    let present = CLOCK.replace(future);
    debug_assert!(present <= future, "Future < Present");
    debug!("Global time now: {future}");
}

pub fn now() -> Jiffies {
    CLOCK.get()
}
