use std::cell::Cell;

thread_local! {
    pub(crate) static TSO: Cell<usize> = Cell::new(0)
}

pub fn global_unique_id() -> usize {
    TSO.replace(TSO.get() + 1)
}

pub(crate) fn drop_tso() {
    TSO.take();
}
