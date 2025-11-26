use std::{cmp::max, collections::HashMap};

use crate::{
    OutgoingMessages,
    communication::{Destination, Message},
    history::ProcessStep,
    metrics::Metrics,
    network_condition::{BandwidthType, Latency, NetworkBoundedMessageQueue},
    process::{ProcessHandle, ProcessId},
    random::{self, Randomizer},
    time::Jiffies,
};

pub struct Simulation<P, M>
where
    P: ProcessHandle<M>,
    M: Message,
{
    latency: Latency,
    procs: HashMap<ProcessId, (P, NetworkBoundedMessageQueue<M>)>,
    metrics: Metrics,
    global_time: Jiffies,
    max_steps: Jiffies,
}

impl<P, M> Simulation<P, M>
where
    P: ProcessHandle<M>,
    M: Message,
{
    pub(crate) fn new(
        seed: random::Seed,
        max_steps: Jiffies,
        max_network_latency: Jiffies,
    ) -> Self {
        Self {
            latency: Latency::new(Randomizer::new(seed), max_network_latency),
            procs: HashMap::new(),
            metrics: Metrics::default(),
            global_time: Jiffies(0),
            max_steps: max_steps,
        }
    }

    pub(crate) fn add_process(&mut self, id: ProcessId, bandwidth: BandwidthType, proc: P) {
        self.procs
            .insert(id, (proc, NetworkBoundedMessageQueue::new(bandwidth)));
    }

    pub fn run(&mut self) -> Metrics {
        self.initial_step();

        while self.keep_running() {
            if !self.step() {
                panic!("Deadlock")
            }
        }

        self.metrics.clone()
    }
}

impl<P, M> Simulation<P, M>
where
    P: ProcessHandle<M>,
    M: Message,
{
    fn submit_messages(&mut self, source: ProcessId, messages: Vec<(Destination, M)>) {
        messages.into_iter().for_each(|(destination, event)| {
            self.submit_event_after(event, source, destination, Jiffies(1));
        });
    }

    fn submit_event_after(
        &mut self,
        message: M,
        source: ProcessId,
        destination: Destination,
        after: Jiffies,
    ) {
        let will_arrive_at = self.calculate_arrival_time(after);

        let targets = match destination {
            Destination::Broadcast => self.procs.keys().copied().collect::<Vec<ProcessId>>(),
            Destination::To(to) => vec![to],
            Destination::SendSelf => vec![source],
        };

        targets.into_iter().for_each(|target| {
            self.devilery_queue_of(target)
                .push((source, message.clone()), will_arrive_at);
        });
    }

    fn devilery_queue_of(&mut self, process_id: ProcessId) -> &mut NetworkBoundedMessageQueue<M> {
        &mut self
            .procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
            .1
    }

    fn handle_of(&mut self, process_id: ProcessId) -> &mut P {
        &mut self
            .procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
            .0
    }

    fn keep_running(&mut self) -> bool {
        self.global_time < self.max_steps
    }

    fn calculate_arrival_time(&mut self, after: Jiffies) -> Jiffies {
        after + self.global_time + self.latency.introduce_random_latency()
    }

    fn initial_step(&mut self) {
        for id in self.procs.keys().copied().collect::<Vec<ProcessId>>() {
            let mut outgoing_messages = OutgoingMessages::new();
            self.procs
                .get_mut(&id)
                .unwrap()
                .0
                .init(&mut outgoing_messages);
            self.submit_messages(id, outgoing_messages.0);
        }
    }

    fn step(&mut self) -> bool {
        let (next_steps, next_time, all_empty) = self.choose_next_processes_steps();
        if all_empty {
            return false;
        }

        // Bandwidth case
        if next_steps.is_empty() {
            return true;
        }

        self.global_time = next_time;
        self.execute_processes_steps(next_steps);
        return true;
    }

    fn execute_processes_steps(&mut self, steps: Vec<ProcessStep<M>>) {
        steps.into_iter().for_each(|(source, message, target)| {
            self.metrics.track_event();
            let mut outgoing_messages = OutgoingMessages::new();
            self.handle_of(target)
                .on_message(source, message, &mut outgoing_messages);
            self.submit_messages(target, outgoing_messages.0);
        })
    }

    fn choose_next_processes_steps(&mut self) -> (Vec<ProcessStep<M>>, Jiffies, bool) {
        let mut all_queues_are_empty = false;
        let mut next_time_pos = Jiffies(0);
        let steps = self
            .procs
            .iter_mut()
            .filter_map(|(candidate, (_, candidate_queue))| {
                all_queues_are_empty |= candidate_queue.is_empty();
                candidate_queue
                    .try_pop(self.global_time)
                    .map(|(time, (source, event))| {
                        next_time_pos = max(time, next_time_pos);
                        (source, event, *candidate)
                    })
            })
            .collect();

        return (steps, next_time_pos, all_queues_are_empty);
    }
}
