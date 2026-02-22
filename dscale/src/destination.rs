use crate::ProcessId;

pub enum Destination {
    BroadcastWithinPool(&'static str),
    To(ProcessId),
}
