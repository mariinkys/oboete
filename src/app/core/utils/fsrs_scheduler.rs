// SPDX-License-Identifier: GPL-3.0

use fsrs::{FSRS, MemoryState, NextStates};

/// Allows us to schedule flashcards using the fsrs algorithm
pub struct FSRSScheduler {
    fsrs: FSRS,
    desired_retention: f32,
}

impl FSRSScheduler {
    /// Init a new [`FSRSScheduler`]
    pub fn new(desired_retention: f32) -> Result<Self, anywho::Error> {
        // Use default parameters (works well for most users)
        let fsrs = FSRS::new(Some(&[]))?;

        Ok(Self {
            fsrs,
            desired_retention,
        })
    }

    // Get next states for a card
    pub fn get_next_states(
        &self,
        memory_state: Option<MemoryState>,
        days_elapsed: u32,
    ) -> Result<NextStates, anywho::Error> {
        Ok(self
            .fsrs
            .next_states(memory_state, self.desired_retention, days_elapsed)?)
    }
}
