use crate::{Jiffies, ProcessId};

pub type Key = String;
pub type Value = String;

#[derive(Clone)]
pub enum SingleKeyOperation {
    Read(Value),
    Write(Value),
}

#[derive(Clone)]
pub struct Entry {
    pub client: ProcessId,
    pub key: Key,
    pub operation: SingleKeyOperation,
    pub result: Option<Value>,
    pub start: Jiffies,
    pub end: Jiffies,
}

pub type ExecutionHistory = Vec<Entry>;
