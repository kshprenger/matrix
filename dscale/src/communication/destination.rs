use crate::ProcessId;

pub enum Destination {
    Broadcast,
    BroadcastWithinPool(&'static str),
    To(ProcessId),
}
