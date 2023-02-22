use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Scheduler {
    scheduler_actions: Vec<String>,
    scheduler_counters: Vec<f32>,
    scheduler_start_counters: Vec<i32>,
}

impl Scheduler {
    pub fn new(
        scheduler_actions: Vec<String>,
        scheduler_counters: Vec<f32>,
        scheduler_start_counters: Vec<i32>,
    ) -> Self {
        Self {
            scheduler_actions,
            scheduler_counters,
            scheduler_start_counters,
        }
    }

    pub fn scheduler_actions(&self) -> Vec<String> {
        self.scheduler_actions.clone()
    }

    pub fn scheduler_counters(&self) -> Vec<f32> {
        self.scheduler_counters.clone()
    }

    pub fn scheduler_start_counters(&self) -> Vec<i32> {
        self.scheduler_start_counters.clone()
    }
}
