use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Pid(pub u32);

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Bytes(pub u64);

impl Bytes {
    pub fn to_kb(&self) -> f64 {
        self.0 as f64 / 1024.0
    }

    pub fn to_mb(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0)
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 < 1024 {
            write!(f, "{} B", self.0)
        } else if self.0 < 1024 * 1024 {
            write!(f, "{:.1} KB", self.to_kb())
        } else {
            write!(f, "{:.1} MB", self.to_mb())
        }
    }
}

impl std::ops::Add for Bytes {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Bytes(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Bytes {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::Sub for Bytes {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Bytes(self.0.saturating_sub(other.0))
    }
}
