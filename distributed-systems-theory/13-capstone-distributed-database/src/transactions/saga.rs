//! Saga-based distributed transaction coordinator.
//!
//! A Saga decomposes a distributed transaction into a sequence of local steps, each
//! with a corresponding compensating action. If any step fails, the Saga executes
//! compensations in reverse order to undo the completed steps, providing eventual
//! atomicity without distributed locking.

/// A single step in a Saga transaction.
#[derive(Debug, Clone)]
pub struct SagaStep {
    /// Human-readable name for this step.
    pub name: String,
    /// Whether this step has already been compensated.
    pub compensated: bool,
}

/// The current status of a Saga transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaStatus {
    /// The Saga is actively executing steps.
    Running,
    /// All steps have completed successfully.
    Completed,
    /// Compensations are in progress.
    Compensating,
    /// All completed steps have been compensated.
    Compensated,
    /// The Saga failed for the given reason and has been compensated.
    Failed(String),
}

/// Coordinates the execution and optional compensation of a Saga transaction.
#[derive(Debug)]
pub struct SagaCoordinator {
    /// The ordered list of steps in this Saga.
    steps: Vec<SagaStep>,
    /// Current status of the Saga.
    status: SagaStatus,
    /// Indices of steps that have been successfully executed.
    completed_indices: Vec<usize>,
}

impl SagaCoordinator {
    /// Create a new Saga coordinator in the Running state.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            status: SagaStatus::Running,
            completed_indices: Vec::new(),
        }
    }

    /// Add a new step to the Saga.
    ///
    /// # Arguments
    ///
    /// * `name` - A human-readable name for the step.
    ///
    /// # Returns
    ///
    /// The index of the newly added step.
    pub fn add_step(&mut self, name: &str) -> usize {
        let index = self.steps.len();
        self.steps.push(SagaStep {
            name: name.to_string(),
            compensated: false,
        });
        index
    }

    /// Mark a step as successfully completed.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the step to mark as completed.
    pub fn execute_step(&mut self, index: usize) -> Result<(), String> {
        if index >= self.steps.len() {
            return Err(format!("step index {} out of range", index));
        }
        self.completed_indices.push(index);
        Ok(())
    }

    /// Compensate all completed steps in reverse order.
    ///
    /// Transitions the Saga through `Compensating` to `Compensated`.
    pub fn compensate(&mut self) {
        self.status = SagaStatus::Compensating;
        for &idx in self.completed_indices.iter().rev() {
            self.steps[idx].compensated = true;
        }
        self.status = SagaStatus::Compensated;
    }

    /// Signal that the Saga has failed and trigger compensation.
    ///
    /// # Arguments
    ///
    /// * `reason` - A description of the failure.
    pub fn fail(&mut self, reason: String) {
        self.compensate();
        self.status = SagaStatus::Failed(reason);
    }

    /// Return the current status of the Saga.
    pub fn get_status(&self) -> &SagaStatus {
        &self.status
    }

    /// Return `true` if all completed steps have been compensated.
    pub fn is_compensated(&self) -> bool {
        self.status == SagaStatus::Compensated
            || matches!(&self.status, SagaStatus::Failed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_execute_steps() {
        let mut saga = SagaCoordinator::new();
        let i0 = saga.add_step("debit");
        let i1 = saga.add_step("credit");
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);

        assert!(saga.execute_step(i0).is_ok());
        assert!(saga.execute_step(i1).is_ok());
        assert_eq!(saga.get_status(), &SagaStatus::Running);
    }

    #[test]
    fn test_compensate() {
        let mut saga = SagaCoordinator::new();
        let i0 = saga.add_step("step1");
        let i1 = saga.add_step("step2");
        saga.execute_step(i0).unwrap();
        saga.execute_step(i1).unwrap();

        saga.compensate();
        assert!(saga.is_compensated());
        assert_eq!(saga.get_status(), &SagaStatus::Compensated);

        // Both steps should be marked compensated.
        assert!(saga.steps[i0].compensated);
        assert!(saga.steps[i1].compensated);
    }

    #[test]
    fn test_fail_triggers_compensation() {
        let mut saga = SagaCoordinator::new();
        let i0 = saga.add_step("step1");
        saga.execute_step(i0).unwrap();

        saga.fail("network error".into());
        assert!(saga.is_compensated());
        assert!(matches!(saga.get_status(), SagaStatus::Failed(_)));
    }

    #[test]
    fn test_execute_step_out_of_range() {
        let mut saga = SagaCoordinator::new();
        assert!(saga.execute_step(0).is_err());
    }
}
