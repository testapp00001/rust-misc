//! # Exercise: Two Generals Simulation
//!
//! ## Theory
//!
//! The Two Generals' Problem illustrates the impossibility of achieving guaranteed
//! agreement between two parties communicating over an unreliable channel. Two
//! generals (processes) must coordinate an attack time, but every message they send
//! may be lost.
//!
//! The protocol proceeds in rounds:
//! 1. General A sends an attack time to General B.
//! 2. General B sends an acknowledgement.
//! 3. General A sends an acknowledgement of the acknowledgement.
//! 4. And so on...
//!
//! The critical observation: **the last message in any finite exchange can always be
//! lost**, leaving one party uncertain whether the other agreed.
//!
//! ## Proof / Intuition
//!
//! Consider a protocol with exactly *k* messages. After the *k*-th message is sent,
//! the sender must commit to the attack (or not). But if that message is lost, the
//! receiver's state after *k-1* messages is identical to the state after *k-1*
//! messages in a protocol that only uses *k-1* messages. By induction, no finite
//! protocol works.
//!
//! In our simulation, we run many trials and observe that some runs never reach
//! agreement, proving that even with retries, the problem persists for any finite
//! number of attempts.
//!
//! ## Implementation Task
//!
//! Implement a simulation where:
//! - Two async tasks represent General A and General B
//! - Communication happens through an unreliable channel
//! - The protocol exchanges messages until agreement or a timeout
//! - Track whether agreement was reached
//!
//! ## Verification
//!
//! - Run many simulations to show that some runs never reach agreement
//! - Verify that the success rate improves with more retries
//! - Show that success rate never reaches 100% (for lossy channels)

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

/// Represents a message exchanged between generals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Propose an attack time (minutes from now).
    AttackTime(u64),
    /// Acknowledge receipt of the attack time.
    Ack,
}

/// Configuration for a Two Generals simulation run.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// The attack time General A proposes (in minutes from now).
    pub proposed_time: u64,
    /// Loss rate for messages sent from A to B.
    pub loss_rate_a_to_b: f64,
    /// Loss rate for messages sent from B to A.
    pub loss_rate_b_to_a: f64,
    /// Maximum number of message rounds before giving up.
    pub max_rounds: u32,
    /// Delay between retries (in milliseconds).
    pub retry_delay_ms: u64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            proposed_time: 10,
            loss_rate_a_to_b: 0.3,
            loss_rate_b_to_a: 0.3,
            max_rounds: 10,
            retry_delay_ms: 5,
        }
    }
}

/// Result of a single simulation run.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Whether both generals reached agreement.
    pub agreement_reached: bool,
    /// Number of message rounds attempted.
    pub rounds_used: u32,
}

/// Simulate a single run of the Two Generals protocol.
///
/// Returns a `SimulationResult` indicating whether agreement was reached.
pub async fn simulate_two_generals(config: SimulationConfig) -> SimulationResult {
    let (tx_a_to_b, mut rx_a_to_b) = mpsc::channel::<Message>(10);
    let (tx_b_to_a, mut rx_b_to_a) = mpsc::channel::<Message>(10);

    let a_agreed = Arc::new(AtomicBool::new(false));
    let b_agreed = Arc::new(AtomicBool::new(false));
    let rounds_used = Arc::new(AtomicUsize::new(0));

    let a_agreed_clone = Arc::clone(&a_agreed);
    let b_agreed_clone = Arc::clone(&b_agreed);
    let rounds_clone = Arc::clone(&rounds_used);
    let config_clone = config.clone();

    // General A task
    let general_a = tokio::spawn(async move {
        let proposed = config_clone.proposed_time;

        for round in 0..config_clone.max_rounds {
            // Send attack time to B
            let msg_sent = send_with_loss(&tx_a_to_b, Message::AttackTime(proposed), config_clone.loss_rate_a_to_b).await;

            if msg_sent {
                // Wait for acknowledgement from B
                let ack_received = wait_for_message(&mut rx_b_to_a, config_clone.retry_delay_ms).await;

                if ack_received.is_some() {
                    // A received B's ack -- A agrees (but B might not get A's ack-of-ack)
                    a_agreed_clone.store(true, Ordering::SeqCst);
                    rounds_clone.store((round + 1) as usize, Ordering::SeqCst);

                    // Send ack-of-ack (but this might be lost, leaving B uncertain)
                    let _ = send_with_loss(&tx_a_to_b, Message::Ack, config_clone.loss_rate_a_to_b).await;
                    return;
                }
            }
        }
        rounds_clone.store(config_clone.max_rounds as usize, Ordering::SeqCst);
    });

    let config_clone = config.clone();

    // General B task
    let general_b = tokio::spawn(async move {
        for _ in 0..config_clone.max_rounds {
            // Wait for attack time from A
            match wait_for_message(&mut rx_a_to_b, config_clone.retry_delay_ms).await {
                Some(Message::AttackTime(_time)) => {
                    // B receives A's proposal -- send ack
                    let _ = send_with_loss(&tx_b_to_a, Message::Ack, config_clone.loss_rate_b_to_a).await;

                    // Wait for ack-of-ack from A (if lost, B is uncertain)
                    let final_ack = wait_for_message(&mut rx_a_to_b, config_clone.retry_delay_ms).await;

                    if let Some(Message::Ack) = final_ack {
                        b_agreed_clone.store(true, Ordering::SeqCst);
                    }
                    // Even if we got the ack-of-ack, we note that B's agreement
                    // is conditional -- in a real scenario, B can't be 100% sure
                    // the final ack arrived. For simulation purposes, we track
                    // whether B received the final confirmation.
                    return;
                }
                _ => continue,
            }
        }
    });

    // Wait for both tasks to complete (with a timeout)
    let _ = tokio::time::timeout(Duration::from_secs(2), async {
        let _ = tokio::join!(general_a, general_b);
    })
    .await;

    SimulationResult {
        agreement_reached: a_agreed.load(Ordering::SeqCst) && b_agreed.load(Ordering::SeqCst),
        rounds_used: rounds_used.load(Ordering::SeqCst) as u32,
    }
}

/// Send a message, randomly dropping it based on loss rate.
async fn send_with_loss(tx: &mpsc::Sender<Message>, msg: Message, loss_rate: f64) -> bool {
    let roll: f64 = rand::random();
    if roll < loss_rate {
        false // Message dropped
    } else {
        tx.send(msg).await.is_ok()
    }
}

/// Wait for a message with a timeout.
async fn wait_for_message(rx: &mut mpsc::Receiver<Message>, timeout_ms: u64) -> Option<Message> {
    tokio::time::timeout(Duration::from_millis(timeout_ms), rx.recv())
        .await
        .unwrap_or(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn zero_loss_always_agrees() {
        let config = SimulationConfig {
            loss_rate_a_to_b: 0.0,
            loss_rate_b_to_a: 0.0,
            max_rounds: 3,
            retry_delay_ms: 1,
            ..Default::default()
        };

        for _ in 0..10 {
            let result = simulate_two_generals(config.clone()).await;
            assert!(
                result.agreement_reached,
                "with 0% loss, agreement should always be reached"
            );
        }
    }

    #[tokio::test]
    async fn high_loss_fails_to_agree() {
        let config = SimulationConfig {
            loss_rate_a_to_b: 0.95,
            loss_rate_b_to_a: 0.95,
            max_rounds: 1,
            retry_delay_ms: 1,
            ..Default::default()
        };

        let mut failures = 0;
        let trials = 100;
        for _ in 0..trials {
            let result = simulate_two_generals(config.clone()).await;
            if !result.agreement_reached {
                failures += 1;
            }
        }

        // With 95% loss and only 1 round, agreement should fail most of the time
        assert!(
            failures > 50,
            "with 95% loss and 1 round, agreement should fail most of the time, got {failures}/{trials} failures"
        );
    }

    #[tokio::test]
    async fn moderate_loss_some_runs_fail() {
        let config = SimulationConfig {
            loss_rate_a_to_b: 0.5,
            loss_rate_b_to_a: 0.5,
            max_rounds: 2,
            retry_delay_ms: 1,
            ..Default::default()
        };

        let mut successes = 0;
        let mut failures = 0;
        let trials = 200;

        for _ in 0..trials {
            let result = simulate_two_generals(config.clone()).await;
            if result.agreement_reached {
                successes += 1;
            } else {
                failures += 1;
            }
        }

        // With 50% loss, some runs should fail
        assert!(
            failures > 0,
            "with 50% loss, at least some runs should fail"
        );
        assert!(
            successes > 0,
            "with 50% loss and retries, some runs should succeed"
        );
    }

    #[tokio::test]
    async fn more_retries_improve_success_rate() {
        let low_retries = SimulationConfig {
            loss_rate_a_to_b: 0.4,
            loss_rate_b_to_a: 0.4,
            max_rounds: 1,
            retry_delay_ms: 1,
            ..Default::default()
        };
        let high_retries = SimulationConfig {
            max_rounds: 8,
            ..low_retries.clone()
        };

        let mut low_successes = 0;
        let mut high_successes = 0;
        let trials = 200;

        for _ in 0..trials {
            if simulate_two_generals(low_retries.clone())
                .await
                .agreement_reached
            {
                low_successes += 1;
            }
            if simulate_two_generals(high_retries.clone())
                .await
                .agreement_reached
            {
                high_successes += 1;
            }
        }

        assert!(
            high_successes >= low_successes,
            "more retries should not decrease success rate"
        );
    }
}
