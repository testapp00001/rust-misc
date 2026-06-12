//! # Exercise: Saga Pattern with Orchestration
//!
//! ## Theory
//!
//! In the orchestration approach, a central coordinator (orchestrator) manages the
//! saga workflow. The orchestrator explicitly tells each service what to do and
//! handles failures by executing compensating transactions.
//!
//! ## Proof / Intuition
//!
//! The orchestrator maintains the workflow definition as a sequence of steps:
//!
//! 1. Step 1: ReserveStock
//!    - Compensation: ReleaseStock
//! 2. Step 2: ProcessPayment
//!    - Compensation: RefundPayment
//! 3. Step 3: CreateOrder
//!    - Compensation: CancelOrder
//! 4. Step 4: SendNotification
//!    - Compensation: (none, best-effort)
//!
//! When a step fails:
//! 1. The orchestrator stops forward progress.
//! 2. It executes compensating transactions in reverse order.
//! 3. It reports the saga as failed.
//!
//! **Advantages of orchestration:**
//! - Clear, centralized workflow definition.
//! - Easy to add new steps or modify the flow.
//! - Services don't need to know about each other.
//!
//! **Disadvantages:**
//! - Single point of failure (the orchestrator).
//! - Orchestrator can become a bottleneck.
//! - Tight coupling between orchestrator and services.
//!
//! ## Implementation Task
//!
//! Implement an orchestrated saga:
//!
//! - `SagaStep`: A step with execute and compensate functions.
//! - `SagaOrchestrator`: manages the sequence of steps.
//! - `SagaError`: error type for saga failures.
//! - `execute(&self) -> Result<(), SagaError>`: run forward.
//! - `compensate(&self)`: run compensations in reverse.
//!
//! ## Verification
//!
//! - Verify happy path completes all steps.
//! - Verify compensation on failure runs in correct reverse order.
//! - Verify partial compensation for partially completed sagas.

use std::collections::HashMap;

/// Errors that can occur during saga execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaError {
    StepFailed { step_name: String, reason: String },
    CompensationFailed { step_name: String, reason: String },
}

/// Result of a saga step execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Success,
    Failed { reason: String },
}

/// A step in the saga with its compensate function.
pub struct SagaStep {
    /// Name of this step.
    pub name: String,
    /// Execute function: returns StepResult.
    pub execute_fn: Box<dyn Fn() -> StepResult>,
    /// Compensate function: returns StepResult.
    pub compensate_fn: Box<dyn Fn() -> StepResult>,
}

impl SagaStep {
    pub fn new(
        name: &str,
        execute_fn: Box<dyn Fn() -> StepResult>,
        compensate_fn: Box<dyn Fn() -> StepResult>,
    ) -> Self {
        Self {
            name: name.to_string(),
            execute_fn,
            compensate_fn,
        }
    }
}

/// Record of a step execution for the log.
#[derive(Debug, Clone)]
pub struct StepRecord {
    pub step_name: String,
    pub result: StepResult,
    pub compensated: bool,
}

/// The saga orchestrator.
pub struct SagaOrchestrator {
    /// Steps to execute in order.
    pub steps: Vec<SagaStep>,
    /// Execution log.
    pub log: Vec<StepRecord>,
    /// Whether the saga has completed.
    pub completed: bool,
    /// Whether compensation was needed.
    pub compensated: bool,
}

impl SagaOrchestrator {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            log: Vec::new(),
            completed: false,
            compensated: false,
        }
    }

    /// Add a step to the saga.
    pub fn add_step(&mut self, step: SagaStep) {
        self.steps.push(step);
    }

    /// Execute the saga forward. If any step fails, compensate and return error.
    pub fn execute(&mut self) -> Result<(), SagaError> {
        let mut completed_count = 0;
        let num_steps = self.steps.len();

        for i in 0..num_steps {
            let result = (self.steps[i].execute_fn)();
            let step_name = self.steps[i].name.clone();
            self.log.push(StepRecord {
                step_name: step_name.clone(),
                result: result.clone(),
                compensated: false,
            });

            match result {
                StepResult::Success => {
                    completed_count += 1;
                }
                StepResult::Failed { reason } => {
                    // Compensation: run compensating transactions in reverse.
                    self.compensate_up_to(completed_count);
                    self.completed = false;
                    self.compensated = true;
                    return Err(SagaError::StepFailed {
                        step_name,
                        reason,
                    });
                }
            }
        }

        self.completed = true;
        Ok(())
    }

    /// Compensate all completed steps in reverse order.
    fn compensate_up_to(&mut self, count: usize) {
        for i in (0..count).rev() {
            let result = (self.steps[i].compensate_fn)();
            self.log.push(StepRecord {
                step_name: self.steps[i].name.clone(),
                result: result.clone(),
                compensated: true,
            });
        }
    }

    /// Get the list of step names that were executed.
    pub fn completed_steps(&self) -> Vec<&str> {
        self.log
            .iter()
            .filter(|r| !r.compensated && r.result == StepResult::Success)
            .map(|r| r.step_name.as_str())
            .collect()
    }

    /// Get the list of step names that were compensated.
    pub fn compensated_steps(&self) -> Vec<&str> {
        self.log
            .iter()
            .filter(|r| r.compensated)
            .map(|r| r.step_name.as_str())
            .collect()
    }
}

/// A simulated service that can be configured to succeed or fail.
pub struct SimulatedService {
    pub name: String,
    pub will_fail: bool,
    pub executed: Vec<String>,
    pub compensated: Vec<String>,
}

impl SimulatedService {
    pub fn new(name: &str, will_fail: bool) -> Self {
        Self {
            name: name.to_string(),
            will_fail,
            executed: Vec::new(),
            compensated: Vec::new(),
        }
    }

    pub fn execute_step(&mut self, step_name: &str) -> StepResult {
        self.executed.push(step_name.to_string());
        if self.will_fail {
            StepResult::Failed {
                reason: format!("{} failed", self.name),
            }
        } else {
            StepResult::Success
        }
    }

    pub fn compensate_step(&mut self, step_name: &str) -> StepResult {
        self.compensated.push(step_name.to_string());
        StepResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(
        name: &str,
        will_fail: bool,
    ) -> SagaStep {
        let n = name.to_string();
        let n2 = name.to_string();
        let will_fail_clone = will_fail;
        SagaStep::new(
            name,
            Box::new(move || {
                if will_fail_clone {
                    StepResult::Failed {
                        reason: format!("{} failed", n),
                    }
                } else {
                    StepResult::Success
                }
            }),
            Box::new(move || {
                // Compensation always succeeds in this test helper.
                StepResult::Success
            }),
        )
    }

    #[test]
    fn happy_path_completes_all_steps() {
        let mut saga = SagaOrchestrator::new();
        saga.add_step(make_step("ReserveStock", false));
        saga.add_step(make_step("ProcessPayment", false));
        saga.add_step(make_step("CreateOrder", false));

        let result = saga.execute();
        assert!(result.is_ok(), "Saga should complete successfully");
        assert!(saga.completed);
        assert!(!saga.compensated);
        assert_eq!(saga.completed_steps(), vec!["ReserveStock", "ProcessPayment", "CreateOrder"]);
    }

    #[test]
    fn compensation_on_failure() {
        let mut saga = SagaOrchestrator::new();
        saga.add_step(make_step("ReserveStock", false));
        saga.add_step(make_step("ProcessPayment", true)); // Fails here.
        saga.add_step(make_step("CreateOrder", false));

        let result = saga.execute();
        assert!(result.is_err(), "Saga should fail when a step fails");
        assert!(saga.compensated);

        match result.unwrap_err() {
            SagaError::StepFailed { step_name, .. } => {
                assert_eq!(step_name, "ProcessPayment");
            }
            _ => panic!("Expected StepFailed error"),
        }

        // ReserveStock should have been compensated.
        assert!(
            saga.compensated_steps().contains(&"ReserveStock"),
            "ReserveStock should be compensated"
        );
    }

    #[test]
    fn compensation_runs_in_reverse_order() {
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let log_clone = log.clone();

        let mut saga = SagaOrchestrator::new();

        // Step 1: succeeds.
        let log1_exec = log.clone();
        let log1_comp = log.clone();
        saga.add_step(SagaStep::new(
            "Step1",
            Box::new(move || {
                log1_exec.lock().unwrap().push("execute_step1".to_string());
                StepResult::Success
            }),
            Box::new(move || {
                log1_comp.lock().unwrap().push("compensate_step1".to_string());
                StepResult::Success
            }),
        ));

        // Step 2: succeeds.
        let log2_exec = log.clone();
        let log2_comp = log.clone();
        saga.add_step(SagaStep::new(
            "Step2",
            Box::new(move || {
                log2_exec.lock().unwrap().push("execute_step2".to_string());
                StepResult::Success
            }),
            Box::new(move || {
                log2_comp.lock().unwrap().push("compensate_step2".to_string());
                StepResult::Success
            }),
        ));

        // Step 3: fails.
        let log3 = log.clone();
        saga.add_step(SagaStep::new(
            "Step3",
            Box::new(move || {
                log3.lock().unwrap().push("execute_step3".to_string());
                StepResult::Failed {
                    reason: "Step3 failed".to_string(),
                }
            }),
            Box::new(move || StepResult::Success),
        ));

        let _ = saga.execute();

        let log_entries = log_clone.lock().unwrap();
        // Compensation should be in reverse order: step2 first, then step1.
        let comp_step2_pos = log_entries.iter().position(|e| e == "compensate_step2");
        let comp_step1_pos = log_entries.iter().position(|e| e == "compensate_step1");

        assert!(comp_step2_pos.is_some(), "Step2 should be compensated");
        assert!(comp_step1_pos.is_some(), "Step1 should be compensated");
        assert!(
            comp_step2_pos.unwrap() < comp_step1_pos.unwrap(),
            "Step2 should be compensated before Step1 (reverse order)"
        );
    }

    #[test]
    fn partial_compensation_for_early_failure() {
        let mut saga = SagaOrchestrator::new();
        saga.add_step(make_step("Step1", false));
        saga.add_step(make_step("Step2", false));
        saga.add_step(make_step("Step3", true)); // Fails at step 3.
        saga.add_step(make_step("Step4", false));

        let _ = saga.execute();

        // Steps 1 and 2 should be compensated. Step 3 failed (not compensated).
        // Step 4 was never executed.
        assert_eq!(saga.compensated_steps(), vec!["Step2", "Step1"]);
    }

    #[test]
    fn empty_saga_succeeds() {
        let mut saga = SagaOrchestrator::new();
        let result = saga.execute();
        assert!(result.is_ok());
        assert!(saga.completed);
    }
}
