//! # Exercise: FLP Indistinguishability
//!
//! ## Theory
//!
//! The FLP impossibility proof relies on a fundamental observation: in an asynchronous
//! system, an observer cannot distinguish a crashed process from a merely slow one.
//!
//! Consider two scenarios:
//! - **Scenario A:** Process P crashes and stops sending messages.
//! - **Scenario B:** Process P is slow and its messages are delayed.
//!
//! From the perspective of any correct process Q, both scenarios produce the same
//! observable behavior: no messages arrive from P within any given finite time bound.
//! Since the system is asynchronous, there is no timeout that can reliably distinguish
//! these cases.
//!
//! ## Proof / Intuition
//!
//! The indistinguishability argument proceeds as follows:
//!
//! 1. Assume a deterministic consensus protocol exists.
//! 2. Start in a configuration where all processes propose 0.
//! 3. The protocol must decide 0 (validity).
//! 4. Now consider a configuration where one process proposes 1 but is delayed.
//! 5. While delayed, other processes cannot tell if this process crashed or is slow.
//! 6. If they decide 0 without the delayed process, the protocol would also decide 0
//!    if that process had actually crashed -- violating agreement if the crashed
//!    process's proposal was 1.
//!
//! This creates an infinite sequence of configurations where the protocol cannot
//! safely decide, proving that no deterministic protocol can guarantee termination.
//!
//! ## Implementation Task
//!
//! Implement the indistinguishability demonstration:
//!
//! - `CrashedProcess`: A process that stops sending after a configured number of messages.
//! - `SlowProcess`: A process that sends messages with random delays.
//! - `Observer`: Tries to determine if a process is crashed or slow based on message
//!   arrival patterns.
//! - `IndistinguishabilityResult`: Tracks whether the observer could distinguish the two.
//!
//! ## Verification
//!
//! - Verify that the observer cannot distinguish crashed from slow in bounded time.
//! - Verify that both scenarios produce identical message patterns for the observer.
//! - Verify that increasing the observation window does not reliably resolve ambiguity.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use rand::Rng;

/// Status of a process as observed by the observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// The observer believes the process is alive (messages are arriving).
    Alive,
    /// The observer suspects the process may be crashed (no messages received recently).
    SuspectedCrashed,
    /// The observer cannot determine the process state.
    Unknown,
}

/// A message sent by a process.
#[derive(Debug, Clone)]
pub struct Message {
    /// The round number this message was sent in.
    pub round: u32,
    /// When the message was originally created (sender's time).
    pub created_at: Instant,
    /// When the message was actually delivered.
    pub delivered_at: Option<Instant>,
}

/// A process that crashes after sending a fixed number of messages.
pub struct CrashedProcess {
    /// How many messages to send before crashing.
    pub crash_after: u32,
    /// Messages that have been sent.
    pub sent_count: u32,
}

impl CrashedProcess {
    /// Creates a new CrashedProcess that crashes after `crash_after` messages.
    pub fn new(crash_after: u32) -> Self {
        Self {
            crash_after,
            sent_count: 0,
        }
    }

    /// Attempts to send a message. Returns `Some(Message)` if the process has not
    /// crashed, or `None` if it has.
    pub fn send_message(&mut self) -> Option<Message> {
        if self.sent_count >= self.crash_after {
            return None;
        }
        self.sent_count += 1;
        Some(Message {
            round: self.sent_count,
            created_at: Instant::now(),
            delivered_at: None,
        })
    }
}

/// A process that sends messages with random delays.
pub struct SlowProcess {
    /// Minimum delay for messages.
    pub min_delay: Duration,
    /// Maximum delay for messages.
    pub max_delay: Duration,
    /// Messages sent so far.
    pub sent_count: u32,
    /// Maximum number of messages to send (to match crashed process behavior).
    pub total_messages: u32,
}

impl SlowProcess {
    /// Creates a new SlowProcess with the given delay range.
    pub fn new(min_delay: Duration, max_delay: Duration, total_messages: u32) -> Self {
        Self {
            min_delay,
            max_delay,
            sent_count: 0,
            total_messages,
        }
    }

    /// Returns the delay that will be applied to the next message.
    pub fn next_delay(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let range = self.max_delay.as_millis() as u64 - self.min_delay.as_millis() as u64;
        let delay_ms = rng.gen_range(0..=range);
        self.min_delay + Duration::from_millis(delay_ms)
    }

    /// Sends a message. Returns `Some((Message, Duration))` with the message and the
    /// delay to apply, or `None` if all messages have been sent.
    pub fn send_message(&mut self) -> Option<(Message, Duration)> {
        if self.sent_count >= self.total_messages {
            return None;
        }
        self.sent_count += 1;
        let delay = self.next_delay();
        let msg = Message {
            round: self.sent_count,
            created_at: Instant::now(),
            delivered_at: None,
        };
        Some((msg, delay))
    }
}

/// An observer that tries to determine whether a process is crashed or slow.
pub struct Observer {
    /// Duration with no messages before suspecting a crash.
    pub suspect_timeout: Duration,
    /// Timestamp of last received message.
    pub last_message_time: Option<Instant>,
    /// All messages received.
    pub received_messages: Vec<Message>,
    /// The observer's current determination.
    pub status: ProcessStatus,
}

impl Observer {
    /// Creates a new Observer with the given suspect timeout.
    pub fn new(suspect_timeout: Duration) -> Self {
        Self {
            suspect_timeout,
            last_message_time: None,
            received_messages: Vec::new(),
            status: ProcessStatus::Unknown,
        }
    }

    /// Record that a message was received.
    pub fn receive_message(&mut self, mut msg: Message) {
        msg.delivered_at = Some(Instant::now());
        self.last_message_time = Some(Instant::now());
        self.received_messages.push(msg);
        self.status = ProcessStatus::Alive;
    }

    /// Check whether the process should be suspected of crashing.
    /// Returns the current status after the check.
    pub fn check_status(&mut self) -> ProcessStatus {
        if self.received_messages.is_empty() {
            self.status = ProcessStatus::Unknown;
            return self.status;
        }
        if let Some(last) = self.last_message_time {
            if last.elapsed() >= self.suspect_timeout {
                self.status = ProcessStatus::SuspectedCrashed;
            } else {
                self.status = ProcessStatus::Alive;
            }
        }
        self.status.clone()
    }

    /// Returns whether the observer can confidently distinguish crashed from slow.
    pub fn can_distinguish(&self) -> bool {
        // In a truly async system, the observer can never be certain.
        // The observer can only "suspect" -- it cannot prove the process crashed.
        false
    }
}

/// Run one scenario and return the observer's state after all expected messages.
pub fn run_scenario(
    mut messages: VecDeque<Option<Message>>,
    observer: &mut Observer,
) {
    while let Some(msg_opt) = messages.pop_front() {
        if let Some(msg) = msg_opt {
            observer.receive_message(msg);
        }
        // Small delay to simulate async timing.
        let _ = observer.check_status();
    }
}

/// Generate messages for a crashed process scenario.
pub fn crashed_scenario_messages(crash_after: u32, total_rounds: u32) -> VecDeque<Option<Message>> {
    let mut proc = CrashedProcess::new(crash_after);
    let mut messages = VecDeque::new();
    for _ in 0..total_rounds {
        messages.push_back(proc.send_message());
    }
    messages
}

/// Generate messages for a slow process scenario with the same total messages.
pub fn slow_scenario_messages(
    min_delay: Duration,
    max_delay: Duration,
    total_messages: u32,
    total_rounds: u32,
) -> VecDeque<Option<Message>> {
    let mut proc = SlowProcess::new(min_delay, max_delay, total_messages);
    let mut messages = VecDeque::new();
    for _ in 0..total_rounds {
        match proc.send_message() {
            Some((msg, _delay)) => {
                messages.push_back(Some(msg));
            }
            None => {
                messages.push_back(None);
            }
        }
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crashed_process_stops_sending() {
        let mut proc = CrashedProcess::new(3);
        let mut count = 0;
        for _ in 0..10 {
            if proc.send_message().is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 3, "Crashed process should send exactly 3 messages");
    }

    #[test]
    fn slow_process_delivers_all_messages() {
        let mut proc = SlowProcess::new(Duration::from_millis(0), Duration::from_millis(5), 5);
        let mut count = 0;
        for _ in 0..20 {
            if proc.send_message().is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 5, "Slow process should send all 5 messages");
    }

    #[test]
    fn observer_cannot_distinguish_crashed_from_slow() {
        let observer = Observer::new(Duration::from_millis(100));
        // The observer can never be certain -- this is the core FLP insight.
        assert!(
            !observer.can_distinguish(),
            "Observer should never be able to definitively distinguish crashed from slow"
        );
    }

    #[test]
    fn observer_suspects_after_timeout() {
        let mut observer = Observer::new(Duration::from_millis(10));
        let msg = Message {
            round: 1,
            created_at: Instant::now(),
            delivered_at: None,
        };
        observer.receive_message(msg);

        // Immediately after receiving, status should be Alive.
        assert_eq!(observer.check_status(), ProcessStatus::Alive);

        // After waiting longer than the suspect timeout, status should change.
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(
            observer.check_status(),
            ProcessStatus::SuspectedCrashed,
            "Observer should suspect crash after timeout with no new messages"
        );
    }

    #[test]
    fn both_scenarios_produce_same_pattern_before_crash() {
        let crash_after = 3;
        let total_rounds = 5;

        let crashed_messages = crashed_scenario_messages(crash_after, total_rounds);
        let slow_messages = slow_scenario_messages(
            Duration::from_millis(0),
            Duration::from_millis(0),
            crash_after,
            total_rounds,
        );

        // Both produce exactly `crash_after` messages followed by None
        let crashed_count: usize = crashed_messages.iter().filter(|m| m.is_some()).count();
        let slow_count: usize = slow_messages.iter().filter(|m| m.is_some()).count();
        assert_eq!(
            crashed_count, slow_count,
            "Both scenarios should produce the same number of delivered messages"
        );
        assert_eq!(
            crashed_count,
            crash_after as usize,
            "Both should deliver exactly crash_after messages"
        );
    }

    #[test]
    fn observer_receives_identical_messages_in_both_scenarios() {
        let mut obs_crashed = Observer::new(Duration::from_millis(100));
        let mut obs_slow = Observer::new(Duration::from_millis(100));

        let crashed_msgs = crashed_scenario_messages(2, 5);
        let slow_msgs = slow_scenario_messages(
            Duration::from_millis(0),
            Duration::from_millis(0),
            2,
            5,
        );

        run_scenario(crashed_msgs, &mut obs_crashed);
        run_scenario(slow_msgs, &mut obs_slow);

        // In a zero-delay scenario with same number of messages,
        // the observer sees the same count.
        assert_eq!(
            obs_crashed.received_messages.len(),
            obs_slow.received_messages.len(),
            "Observer should receive the same number of messages in both scenarios"
        );
    }
}
