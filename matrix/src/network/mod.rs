mod bandwidth;
mod latency;

use std::cell::RefCell;
use std::rc::Rc;

pub use bandwidth::BandwidthDescription;
pub(crate) use bandwidth::BandwidthQueue;
pub(crate) use latency::LatencyQueue;
use log::debug;

use crate::Destination;
use crate::Message;
use crate::MessagePtr;
use crate::Now;
use crate::ProcessId;
use crate::actor::SimulationActor;
use crate::communication::ProcessStep;
use crate::communication::RoutedMessage;
use crate::global;
use crate::global::configuration;
use crate::random::Randomizer;
use crate::random::Seed;
use crate::time::Jiffies;
use crate::topology::Topology;

pub(crate) type NetworkActor = Rc<RefCell<Network>>;

pub(crate) struct Network {
    seed: Seed,
    bandwidth_queue: BandwidthQueue,
    topology: Rc<Topology>,
}

impl Network {
    fn SubmitSingleMessage(
        &mut self,
        message: Rc<dyn Message>,
        source: ProcessId,
        destination: Destination,
    ) {
        let targets = match destination {
            Destination::Broadcast => self.topology.Keys().copied().collect::<Vec<ProcessId>>(),
            Destination::BroadcastWithinPool(pool_name) => {
                self.topology.ListPool(pool_name).to_vec()
            }
            Destination::To(to) => vec![to],
        };

        debug!("Submitting message from {source}, targets of the message: {targets:?}",);

        targets.into_iter().for_each(|target| {
            let routed_message = RoutedMessage {
                arrival_time: Now() + Jiffies(1), // Without any latency message will arrive on next timepoint;
                step: ProcessStep {
                    source,
                    dest: target,
                    message: message.clone(),
                },
            };
            self.bandwidth_queue.Push(routed_message);
        });
    }

    fn ExecuteProcessStep(&mut self, step: ProcessStep) {
        let source = step.source;
        let dest = step.dest;
        let message = step.message;

        debug!(
            "Executing step for process {} | Message Source: {}",
            dest, source
        );

        global::SetProcess(dest);

        self.topology
            .Get(dest)
            .OnMessage(source, MessagePtr::New(message));
    }
}

impl Network {
    pub(crate) fn New(
        seed: Seed,
        bandwidth_type: BandwidthDescription,
        topology: Rc<Topology>,
    ) -> Self {
        Self {
            seed,
            bandwidth_queue: BandwidthQueue::New(
                bandwidth_type,
                topology.Size(),
                LatencyQueue::New(Randomizer::New(seed), topology.clone()),
            ),
            topology,
        }
    }

    pub(crate) fn SubmitMessages(
        &mut self,
        messages: &mut Vec<(ProcessId, Destination, Rc<dyn Message>)>,
    ) {
        messages
            .drain(..)
            .into_iter()
            .for_each(|(from, destination, message)| {
                self.SubmitSingleMessage(message, from, destination);
            });
    }

    pub(crate) fn GetAvgTotalPasedBytes(&self) -> usize {
        self.bandwidth_queue.GetAvgTotalPasedBytes()
    }
}

impl SimulationActor for Network {
    fn Start(&mut self) {
        self.topology.IterMut().for_each(|(id, mut handle)| {
            debug!("Executing initial step for {id}");

            configuration::SetupLocalConfiguration(*id, self.seed);

            global::SetProcess(*id);

            handle.Start();
        });
    }

    fn Step(&mut self) {
        let next_event = self.bandwidth_queue.Pop();

        match next_event {
            None => {}
            Some(message) => {
                self.ExecuteProcessStep(message.step);
            }
        }
    }

    fn PeekClosest(&self) -> Option<Jiffies> {
        self.bandwidth_queue.PeekClosest()
    }
}
