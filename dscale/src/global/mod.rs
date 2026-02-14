mod access;
pub mod anykv;
pub(crate) mod clock;
pub mod configuration;
pub mod tso;

pub use tso::global_unique_id;

pub use clock::now;

pub use access::broadcast;
pub use access::broadcast_within_pool;
pub use access::choose_from_pool;
pub use access::list_pool;
pub use access::rank;
pub use access::schedule_timer_after;
pub use access::send_random_from_pool;
pub use access::send_to;

pub(crate) use access::schedule;
pub(crate) use access::set_process;
pub(crate) use access::setup_access;

pub(crate) use clock::fast_forward_clock;

pub(crate) fn drop_all() {
    clock::drop_clock();
    tso::drop_tso();
    anykv::drop_anykv();
    access::drop_access();
}
