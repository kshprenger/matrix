use dscale::ProcessId;

pub type Value = usize;
pub type Key = usize;
pub type Timestamp = usize;
pub type ReadSequence = usize;
pub type ClientId = ProcessId;

pub const REPLICA_POOL_NAME: &str = "Replicas";
pub const CLIENT_POOL_NAME: &str = "Clients";
