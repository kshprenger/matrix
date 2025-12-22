use std::{collections::BTreeMap, process::exit, rc::Rc};

use log::{debug, info};

use crate::{
    Access, MessagePtr,
    access::{DrainMessages, SetupAccess},
    communication::{Destination, Message, ProcessStep, RoutedMessage},
    metrics::Metrics,
    network_condition::{BandwidthQueue, BandwidthQueueOptions, BandwidthType, LatencyQueue},
    process::{Configuration, ProcessHandle, ProcessId},
    progress::Bar,
    random::{self, Randomizer},
    time::Jiffies,
};

pub struct Simulation<P>
where
    P: ProcessHandle,
{
    bandwidth_queue: BandwidthQueue,
    procs: BTreeMap<ProcessId, P>,
    metrics: Metrics,
    global_time: Jiffies,
    max_time: Jiffies,
    progress_bar: Bar,
}

const K_PROGRESS_TIMES: usize = 10;

impl<P> Simulation<P>
where
    P: ProcessHandle,
{
    pub(crate) fn New(
        seed: random::Seed,
        max_time: Jiffies,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: Vec<(ProcessId, P)>,
    ) -> Self {
        let _ = env_logger::try_init();

        Self {
            bandwidth_queue: BandwidthQueue::New(
                bandwidth_type,
                procs.len(),
                LatencyQueue::New(Randomizer::New(seed), max_network_latency),
            ),
            procs: procs.into_iter().collect(),
            metrics: Metrics::default(),
            global_time: Jiffies(0),
            max_time: max_time,
            progress_bar: Bar::New(max_time, max_time.0 / K_PROGRESS_TIMES),
        }
    }

    pub fn Run(&mut self) -> Metrics {
        self.InitialStep();

        while self.KeepRunning() {
            if !self.Step() {
                info!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                exit(1)
            }
        }

        self.progress_bar.MakeProgress(self.max_time);

        info!("Looks good! ヽ(‘ー`)ノ");
        self.metrics.clone()
    }
}

impl<P> Simulation<P>
where
    P: ProcessHandle,
{
    fn SubmitMessages(&mut self, source: ProcessId, messages: Vec<(Destination, Rc<dyn Message>)>) {
        messages.into_iter().for_each(|(destination, event)| {
            self.SubmitSingleMessage(event, source, destination, self.global_time + Jiffies(1));
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

    fn KeepRunning(&mut self) -> bool {
        self.global_time < self.max_time
    }

    fn InitialStep(&mut self) {
        for id in self.procs.keys().copied().collect::<Vec<ProcessId>>() {
            debug!("Executing initial step for {id}");
            let config = Configuration {
                assigned_id: id,
                proc_num: self.procs.keys().len(),
            };
            SetupAccess(Access::New(self.global_time));
            self.HandleOf(id).Bootstrap(config);
            self.SubmitMessages(id, DrainMessages());
        }
    }

    fn Step(&mut self) -> bool {
        let next_event = self.bandwidth_queue.Pop();

        match next_event {
            BandwidthQueueOptions::None => false,
            BandwidthQueueOptions::MessageArrivedByLatency => true, // Do nothing
            BandwidthQueueOptions::Some(message) => {
                self.FastForwardClock(message.arrival_time);
                self.ExecuteProcessStep(message.step);
                true
            }
        }
    }

    fn FastForwardClock(&mut self, time: Jiffies) {
        debug_assert!(self.global_time <= time, "Time is not monotonous");
        self.global_time = time;
        self.progress_bar.MakeProgress(time);
        debug!("Global time now: {time}");
    }

    fn ExecuteProcessStep(&mut self, step: ProcessStep) {
        self.metrics.TrackEvent();

        let source = step.source;
        let dest = step.dest;
        let message = step.message;

        debug!(
            "Executing step for process {} | Message Source: {}",
            dest, source
        );

        SetupAccess(Access::New(self.global_time));
        self.HandleOf(dest)
            .OnMessage(source, MessagePtr::New(message));
        self.SubmitMessages(dest, DrainMessages());
    }
}
