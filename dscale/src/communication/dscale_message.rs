use crate::{MessagePtr, TimerId};

pub(crate) enum DScaleMessage {
    NetworkMessage(MessagePtr),
    Timer(TimerId),
}
