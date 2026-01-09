mod access;
pub mod anykv;
pub(crate) mod clock;
pub mod configuration;
pub mod tso;

pub use tso::GlobalUniqueId;

pub use clock::Now;

pub use access::Broadcast;
pub use access::BroadcastWithinPool;
pub use access::CurrentId;
pub use access::ListPool;
pub use access::ScheduleTimerAfter;
pub use access::SendTo;

pub(crate) use access::Drain;
pub(crate) use access::SetProcess;
pub(crate) use access::SetupAccess;

pub(crate) use clock::FastForwardClock;

pub(crate) fn ResetGlobals() {
    clock::Reset();
    tso::Reset();
    anykv::Clear();
}
