use std::cell::Cell;

thread_local! {
    pub(crate) static TSO: Cell<usize> = Cell::new(0)
}

pub fn GlobalUniqueId() -> usize {
    TSO.replace(TSO.get() + 1)
}

pub(crate) fn Drop() {
    TSO.take();
}
