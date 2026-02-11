#![allow(non_snake_case)]

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
    fn Start(&mut self) {
        Debug!("Starting timer demo process");

        // Schedule a heartbeat timer to fire every 1000 jiffies
        let timer_id = ScheduleTimerAfter(Jiffies(1000));
        self.heartbeat_timer = Some(timer_id);
        Debug!(
            "Scheduled heartbeat timer {} to fire in 1000 jiffies",
            timer_id
        );

        // Process 1 starts by sending a ping
        if Rank() == 1 {
            SendTo(2, LazyPingPongMessage::Ping);
        }
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        let m = message.As::<LazyPingPongMessage>();

        match m.as_ref() {
            LazyPingPongMessage::Ping => {
                Debug!("Received Ping from Process {}", from);
                anykv::Modify::<usize>("pings_received", |count| *count += 1);

                // Schedule a delayed response using a timer
                let timer_id = ScheduleTimerAfter(Jiffies(500));
                Debug!("Scheduling delayed pong response with timer {}", timer_id);
            }

            LazyPingPongMessage::DelayedPong => {
                Debug!("Received DelayedPong from Process {}", from);
                anykv::Modify::<usize>("pongs_received", |count| *count += 1);

                // Send another ping if we haven't reached the limit
                self.ping_count += 1;
                if self.ping_count < 5 {
                    SendTo(from, LazyPingPongMessage::Ping);
                }
            }
        }
    }

    fn OnTimer(&mut self, timer_id: TimerId) {
        Debug!("Timer {} fired", timer_id);

        // Check if this is the heartbeat timer
        if let Some(heartbeat_id) = self.heartbeat_timer {
            if timer_id == heartbeat_id {
                Debug!("Heartbeat timer fired");
                anykv::Modify::<usize>("heartbeats", |count| *count += 1);

                // Reschedule the heartbeat timer for continuous operation
                let new_timer_id = ScheduleTimerAfter(Jiffies(1000));
                self.heartbeat_timer = Some(new_timer_id);
                return;
            }
        }

        // This must be a delayed response timer
        Debug!("Delayed response timer fired - sending DelayedPong");
        if Rank() == 2 {
            SendTo(1, LazyPingPongMessage::DelayedPong);
        }
    }
}
