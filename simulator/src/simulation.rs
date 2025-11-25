use std::collections::{HashMap, HashSet};

use crate::{
    communication::{Destination, Event},
    history::ProcessStep,
    metrics::Metrics,
    network_condition::{BandwidthType, Latency, NetworkBoundedQueue},
    process::{ProcessHandle, ProcessId},
    random::{self, Randomizer},
    time::Jiffies,
};

pub struct Simulation {
    latency: Latency,
    procs: HashMap<ProcessId, (Box<dyn ProcessHandle>, NetworkBoundedQueue)>,
    metrics: Metrics,
    global_time: Jiffies,
    max_steps: Jiffies,
}

impl Simulation {
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

    pub(crate) fn submit_event_set(
        &mut self,
        source: ProcessId,
        set: HashSet<(Destination, Event)>,
    ) {
        set.into_iter().for_each(|(destination, event)| {
            self.submit_event_after(event, source, destination, Jiffies(1));
        });
    }

    pub(crate) fn submit_event_after(
        &mut self,
        event: Event,
        source: ProcessId,
        destination: Destination,
        after: Jiffies,
    ) {
        let will_arrive_at = self.calculate_arrival_time(after);

        let targets = match destination {
            Destination::Broadcast => self.procs.keys().copied().collect::<Vec<ProcessId>>(),
            Destination::SendSelf => vec![source],
        };

        targets.into_iter().for_each(|target| {
            self.devilery_queue_of(target)
                .push((source, event.clone()), will_arrive_at);
        });
    }

    pub(crate) fn add_process(
        &mut self,
        id: ProcessId,
        bandwidth: BandwidthType,
        proc: Box<dyn ProcessHandle>,
    ) {
        self.procs
            .insert(id, (proc, NetworkBoundedQueue::new(bandwidth)));
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

impl Simulation {
    fn devilery_queue_of(&mut self, process_id: ProcessId) -> &mut NetworkBoundedQueue {
        &mut self
            .procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
            .1
    }

    fn handle_of(&mut self, process_id: ProcessId) -> &mut Box<dyn ProcessHandle> {
        &mut self
            .procs
            .get_mut(&process_id)
            .expect("Invalid proccess id")
            .0
    }

    fn keep_running(&mut self) -> bool {
        self.tick();
        self.global_time < self.max_steps
    }

    fn tick(&mut self) {
        self.global_time += 1;
    }

    fn calculate_arrival_time(&mut self, after: Jiffies) -> Jiffies {
        after + self.global_time + self.latency.introduce_random_latency()
    }

    fn initial_step(&mut self) {
        for id in self.procs.keys().copied().collect::<Vec<ProcessId>>() {
            let next_events = self.procs.get_mut(&id).unwrap().0.init();
            self.submit_event_set(id, next_events);
        }
    }

    fn step(&mut self) -> bool {
        if self.there_is_no_steps() {
            return false;
        }

        let next_steps = self.choose_next_processes_steps();

        if next_steps.is_empty() {
            // There is steps, but they have greater arrival time than current global time.
            // So this simulation step executes as no-op
            return true;
        }

        self.execute_processes_steps(next_steps);
        return true;
    }

    fn execute_processes_steps(&mut self, steps: Vec<ProcessStep>) {
        steps.into_iter().for_each(|(source, event, target)| {
            self.metrics.track_event();
            let next_events = self.handle_of(target).on_event((source, event));
            self.submit_event_set(target, next_events);
        })
    }

    fn there_is_no_steps(&self) -> bool {
        self.procs.iter().all(|(_, (_, queue))| queue.is_empty())
    }

    fn choose_next_processes_steps(&mut self) -> Vec<ProcessStep> {
        self.procs
            .iter_mut()
            .filter(|(_, (_, candidate_queue))| {
                candidate_queue
                    .peek()
                    .map(|(_, next_event_arrival_time)| {
                        *next_event_arrival_time == self.global_time
                    })
                    .unwrap_or(false)
            })
            .filter_map(|(candidate, (_, candidate_queue))| {
                candidate_queue
                    .try_pop(self.global_time)
                    .map(|(source, event)| (source, event, *candidate))
            })
            .collect()
    }
}
