use dscale::{global::anykv, *};

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum LazyPingPongMessage {
    Ping,
    DelayedPong,
}

impl Message for LazyPingPongMessage {}

#[derive(Default)]
pub struct LazyPingPong {
    heartbeat_timer: Option<TimerId>,
    ping_count: usize,
}

impl ProcessHandle for LazyPingPong {
    fn start(&mut self) {
        debug_process!("Starting timer demo process");

        // Schedule a heartbeat timer to fire every 1000 jiffies
        let timer_id = schedule_timer_after(Jiffies(1000));
        self.heartbeat_timer = Some(timer_id);
        debug_process!(
            "Scheduled heartbeat timer {} to fire in 1000 jiffies",
            timer_id
        );

        // Process 1 starts by sending a ping
        if rank() == 1 {
            send_to(2, LazyPingPongMessage::Ping);
        }
    }

    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        let m = message.as_type::<LazyPingPongMessage>();

        match m.as_ref() {
            LazyPingPongMessage::Ping => {
                debug_process!("Received Ping from Process {}", from);
                anykv::modify::<usize>("pings_received", |count| *count += 1);

                // Schedule a delayed response using a timer
                let timer_id = schedule_timer_after(Jiffies(500));
                debug_process!("Scheduling delayed pong response with timer {}", timer_id);
            }

            LazyPingPongMessage::DelayedPong => {
                debug_process!("Received DelayedPong from Process {}", from);
                anykv::modify::<usize>("pongs_received", |count| *count += 1);

                // Send another ping if we haven't reached the limit
                self.ping_count += 1;
                if self.ping_count < 5 {
                    send_to(from, LazyPingPongMessage::Ping);
                }
            }
        }
    }

    fn on_timer(&mut self, timer_id: TimerId) {
        debug_process!("Timer {} fired", timer_id);

        // Check if this is the heartbeat timer
        if let Some(heartbeat_id) = self.heartbeat_timer {
            if timer_id == heartbeat_id {
                debug_process!("Heartbeat timer fired");
                anykv::modify::<usize>("heartbeats", |count| *count += 1);

                // Reschedule the heartbeat timer for continuous operation
                let new_timer_id = schedule_timer_after(Jiffies(1000));
                self.heartbeat_timer = Some(new_timer_id);
                return;
            }
        }

        // This must be a delayed response timer
        debug_process!("Delayed response timer fired - sending DelayedPong");
        if rank() == 2 {
            send_to(1, LazyPingPongMessage::DelayedPong);
        }
    }
}
