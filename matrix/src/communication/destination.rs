use crate::ProcessId;

pub enum Destination {
    Broadcast,
    BroadcastWithingPool(&'static str),
    To(ProcessId),
}
