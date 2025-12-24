mod access;
mod bandwidth;
mod latency;

use std::collections::BTreeMap;
use std::rc::Rc;

pub use access::Broadcast;
pub use access::SendSelf;
pub use access::SendTo;

pub(crate) use bandwidth::BandwidthQueue;
pub(crate) use bandwidth::BandwidthQueueOptions;
pub use bandwidth::BandwidthType;
pub(crate) use latency::LatencyQueue;
use log::debug;

use crate::Configuration;
use crate::Destination;
use crate::Message;
use crate::MessagePtr;
use crate::ProcessHandle;
use crate::ProcessId;
use crate::communication::ProcessStep;
use crate::communication::RoutedMessage;
use crate::network::access::CreateAccess;
use crate::network::access::DrainMessages;
use crate::random::Randomizer;
use crate::random::Seed;
use crate::time::FastForwardClock;
use crate::time::Jiffies;
use crate::time::Now;

pub(crate) struct Network<P: ProcessHandle> {
    bandwidth_queue: BandwidthQueue,
    procs: BTreeMap<ProcessId, P>,
}

impl<P: ProcessHandle> Network<P> {
    fn SubmitMessages(&mut self, source: ProcessId, messages: Vec<(Destination, Rc<dyn Message>)>) {
        messages.into_iter().for_each(|(destination, event)| {
            self.SubmitSingleMessage(event, source, destination, Now() + Jiffies(1));
        });
    }

    fn SubmitSingleMessage(
        &mut self,
        message: Rc<dyn Message>,
        source: ProcessId,
        destination: Destination,
        base_arrival_time: Jiffies,
    ) {
        let targets = match destination {
            Destination::Broadcast => self.procs.keys().copied().collect::<Vec<ProcessId>>(),
            Destination::To(to) => vec![to],
            Destination::SendSelf => vec![source],
        };

        debug!("Submitting message, targets of the message: {targets:?}",);

        targets.into_iter().for_each(|target| {
            let routed_message = RoutedMessage {
                arrival_time: base_arrival_time,
                step: ProcessStep {
                    source,
                    dest: target,
                    message: message.clone(),
                },
            };
            self.bandwidth_queue.Push(routed_message);
        });
    }

    fn HandleOf(&mut self, process_id: ProcessId) -> &mut P {
        self.procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
    }

    fn ExecuteProcessStep(&mut self, step: ProcessStep) {
        let source = step.source;
        let dest = step.dest;
        let message = step.message;

        debug!(
            "Executing step for process {} | Message Source: {}",
            dest, source
        );

        self.HandleOf(dest)
            .OnMessage(source, MessagePtr::New(message));
        self.SubmitMessages(dest, DrainMessages());
    }
}

impl<P: ProcessHandle> Network<P> {
    pub(crate) fn New(
        seed: Seed,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: BTreeMap<ProcessId, P>,
    ) -> Self {
        Self {
            bandwidth_queue: BandwidthQueue::New(
                bandwidth_type,
                procs.len(),
                LatencyQueue::New(Randomizer::New(seed), max_network_latency),
            ),
            procs,
        }
    }

    pub(crate) fn Start(&mut self) {
        CreateAccess();

        for id in self.procs.keys().copied().collect::<Vec<ProcessId>>() {
            debug!("Executing initial step for {id}");
            let config = Configuration {
                assigned_id: id,
                proc_num: self.procs.keys().len(),
            };

            self.HandleOf(id).Bootstrap(config);
            self.SubmitMessages(id, DrainMessages());
        }
    }

    pub(crate) fn Step(&mut self) -> bool {
        let next_event = self.bandwidth_queue.Pop();

        match next_event {
            BandwidthQueueOptions::None => false,
            BandwidthQueueOptions::MessageArrivedByLatency => true, // Do nothing
            BandwidthQueueOptions::Some(message) => {
                FastForwardClock(message.arrival_time);
                self.ExecuteProcessStep(message.step);
                true
            }
        }
    }
}
