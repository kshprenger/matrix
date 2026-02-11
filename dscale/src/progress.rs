use indicatif::{ProgressBar, ProgressStyle};
use log::log_enabled;

use crate::time::Jiffies;

const K_PROGRESS_TIMES: usize = 20;

pub(crate) struct Bar {
    bar: ProgressBar,
    prev_log: usize,
    delta: usize,
}

impl Bar {
    pub(crate) fn New(total: Jiffies) -> Self {
        let bar = if log_enabled!(log::Level::Info) {
            let bar = ProgressBar::new(total.0 as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:60.green}] {pos}/{len} Jiffies")
                    .unwrap(),
            );
            bar.set_position(0);
            bar
        } else {
            ProgressBar::hidden()
        };

        Self {
            bar: bar,
            prev_log: 0,
            delta: total.0 / K_PROGRESS_TIMES,
        }
    }

    pub(crate) fn MakeProgress(&mut self, time: Jiffies) {
        let d = time.0 / self.delta;
        if d > self.prev_log {
            self.prev_log = d;
            self.bar.set_position(time.0 as u64)
        }
    }

    pub(crate) fn Finish(&mut self) {
        self.bar.finish();
    }
}
