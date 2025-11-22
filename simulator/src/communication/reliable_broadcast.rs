use crate::{
    communication::{Destination, EventType, Message},
    simulation_handle::with_sim,
    time::Jiffies,
};

pub fn r_bcast(message: Message) {
    with_sim(|sim| {
        sim.submit_event_after(
            EventType::Message(message),
            Destination::Broadcast,
            Jiffies(1),
        )
    });
}
