use std::{
    cell::Cell,
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul, Sub},
};

use log::debug;

#[derive(PartialEq, PartialOrd, Ord, Eq, Copy, Clone)]
pub struct Jiffies(pub usize);

impl Add for Jiffies {
    type Output = Jiffies;

    fn add(self, rhs: Self) -> Self::Output {
        Jiffies(self.0 + rhs.0)
    }
}

impl Sub for Jiffies {
    type Output = Jiffies;

    fn sub(self, rhs: Self) -> Self::Output {
        Jiffies(self.0 - rhs.0)
    }
}

impl AddAssign<Jiffies> for Jiffies {
    fn add_assign(&mut self, rhs: Jiffies) {
        self.0 += rhs.0
    }
}

impl AddAssign<usize> for Jiffies {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Mul<Jiffies> for usize {
    type Output = Self;

    fn mul(self, rhs: Jiffies) -> Self::Output {
        self * rhs.0
    }
}

impl Display for Jiffies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(("Jiffies(".to_string() + &self.0.to_string() + ")").as_str())
    }
}

impl Debug for Jiffies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

thread_local! {
    pub(crate) static CLOCK: Cell<Jiffies> = Cell::new(Jiffies(0))
}

pub(crate) fn FastForwardClock(time: Jiffies) {
    CLOCK.with(|cell| {
        let now = cell.get();
        debug_assert!(now <= time, "Time is not monotonous");
        cell.set(time);
        debug!("Global time now: {time}");
    });
}

pub fn Now() -> Jiffies {
    CLOCK.with(|cell| cell.get())
}
