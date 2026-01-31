use std::rc::Rc;

use matrix::{Message, ProcessId};

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct BCBMessageId {
    pub(super) process_id: ProcessId,
    pub(super) message_id: usize,
}

pub enum BCBMessage {
    Initiate((BCBMessageId, Rc<dyn Message>)),
    Signature(BCBMessageId),
    Certificate(usize, BCBMessageId),
}

pub const ID_SIZE: usize = 128;
pub const SIG_SIZE: usize = 64; // For example Ed25519 or Secp256k1

impl Message for BCBMessage {
    fn VirtualSize(&self) -> usize {
        match self {
            BCBMessage::Initiate((_, m)) => ID_SIZE + m.VirtualSize(),
            BCBMessage::Signature(_) => SIG_SIZE,
            BCBMessage::Certificate(k_validators, _) => ID_SIZE + (k_validators / 8),
        }
    }
}
