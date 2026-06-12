//! # Exercise: PACELC Analysis
//!
//! ## Theory
//!
//! The PACELC theorem (Abadi, 2010) extends CAP by addressing the trade-off
//! during normal operation (no partition):
//!
//! - **If Partition:** Choose between **A**vailability and **C**onsistency.
//! - **Else:** Choose between **L**atency and **C**onsistency.
//!
//! This gives four main classifications:
//! - **PA/EL:** Partition: choose Availability; Else: choose Latency.
//!   (e.g., Cassandra, DynamoDB)
//! - **PA/EC:** Partition: choose Availability; Else: choose Consistency.
//!   (rare in practice)
//! - **PC/EL:** Partition: choose Consistency; Else: choose Latency.
//!   (e.g., MongoDB with majority reads)
//! - **PC/EC:** Partition: choose Consistency; Else: choose Consistency.
//!   (e.g., etcd, ZooKeeper, Spanner)
//!
//! ## Proof / Intuition
//!
//! PACELC provides a more complete picture than CAP alone. CAP only tells us
//! what happens during partitions, but most of the time the network is healthy.
//! PACELC captures the latency/consistency trade-off that operators face
//! during normal operation.
//!
//! For example, Cassandra is PA/EL: during partitions it favors availability
//! (accepts writes on any node), and during normal operation it favors low
//! latency (read/write from the local replica without waiting for quorum).
//!
//! etcd is PC/EC: during partitions it rejects writes that can't achieve
//! quorum (favoring consistency), and during normal operation it requires
//! quorum for both reads and writes (favoring consistency over latency).
//!
//! ## Implementation Task
//!
//! Implement a `PacelcAnalysis` that:
//! - Classifies distributed systems by their PACELC trade-off
//! - Provides reasoning for each classification
//! - Compares systems along both axes
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Cassandra is correctly classified as PA/EL
//! - etcd is correctly classified as PC/EC
//! - The analysis framework handles all four PACELC categories

/// Partition behavior during a network partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionBehavior {
    /// Favor Availability: accept writes even during partition.
    PA,
    /// Favor Consistency: reject writes during partition.
    PC,
}

/// Normal behavior when there is no partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalBehavior {
    /// Favor Latency: serve reads/writes from local replica.
    EL,
    /// Favor Consistency: coordinate across replicas.
    EC,
}

/// PACELC classification of a distributed system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacelcClassification {
    pub partition: PartitionBehavior,
    pub normal: NormalBehavior,
}

impl PacelcClassification {
    pub fn new(partition: PartitionBehavior, normal: NormalBehavior) -> Self {
        Self { partition, normal }
    }

    /// Format as the standard PACELC notation string (e.g., "PA/EL").
    pub fn notation(&self) -> String {
        let p = match self.partition {
            PartitionBehavior::PA => "PA",
            PartitionBehavior::PC => "PC",
        };
        let n = match self.normal {
            NormalBehavior::EL => "EL",
            NormalBehavior::EC => "EC",
        };
        format!("{p}/{n}")
    }
}

impl std::fmt::Display for PacelcClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.notation())
    }
}

/// Description and classification of a distributed system.
#[derive(Debug, Clone)]
pub struct SystemAnalysis {
    pub name: String,
    pub classification: PacelcClassification,
    pub description: String,
}

impl SystemAnalysis {
    pub fn new(
        name: &str,
        classification: PacelcClassification,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            classification,
            description: description.to_string(),
        }
    }
}

/// PACELC analysis framework for classifying distributed systems.
pub struct PacelcAnalysis {
    systems: Vec<SystemAnalysis>,
}

impl PacelcAnalysis {
    /// Create a new analysis with pre-classified real-world systems.
    pub fn new() -> Self {
        let systems = vec![
            SystemAnalysis::new(
                "Cassandra",
                PacelcClassification::new(PartitionBehavior::PA, NormalBehavior::EL),
                "Accepts writes on any node during partition. Normal operation \
                 reads from local replica for low latency.",
            ),
            SystemAnalysis::new(
                "DynamoDB",
                PacelcClassification::new(PartitionBehavior::PA, NormalBehavior::EL),
                "Default: accepts writes during partition. Eventually consistent \
                 reads from local replica.",
            ),
            SystemAnalysis::new(
                "etcd",
                PacelcClassification::new(PartitionBehavior::PC, NormalBehavior::EC),
                "Rejects writes without quorum during partition. Requires quorum \
                 for reads and writes during normal operation.",
            ),
            SystemAnalysis::new(
                "ZooKeeper",
                PacelcClassification::new(PartitionBehavior::PC, NormalBehavior::EC),
                "Requires majority for writes during partition. Linearizable reads \
                 during normal operation.",
            ),
            SystemAnalysis::new(
                "Spanner",
                PacelcClassification::new(PartitionBehavior::PC, NormalBehavior::EC),
                "Rejects writes without majority during partition. TrueTime \
                 provides globally consistent reads.",
            ),
            SystemAnalysis::new(
                "MongoDB",
                PacelcClassification::new(PartitionBehavior::PC, NormalBehavior::EL),
                "With majority read concern: PC. Normal reads from primary \
                 for low latency: EL.",
            ),
        ];
        Self { systems }
    }

    /// Register a custom system for classification.
    pub fn register(
        &mut self,
        name: &str,
        classification: PacelcClassification,
        description: &str,
    ) {
        self.systems.push(SystemAnalysis::new(
            name,
            classification,
            description,
        ));
    }

    /// Look up a system by name.
    pub fn lookup(&self, name: &str) -> Option<&SystemAnalysis> {
        self.systems.iter().find(|s| s.name == name)
    }

    /// Get the classification for a system.
    pub fn classify(&self, name: &str) -> Option<PacelcClassification> {
        self.lookup(name).map(|s| s.classification)
    }

    /// Get all systems in a given PACELC category.
    pub fn systems_in_category(
        &self,
        partition: PartitionBehavior,
        normal: NormalBehavior,
    ) -> Vec<&SystemAnalysis> {
        self.systems
            .iter()
            .filter(|s| {
                s.classification.partition == partition
                    && s.classification.normal == normal
            })
            .collect()
    }

    /// Get all registered systems.
    pub fn all_systems(&self) -> &[SystemAnalysis] {
        &self.systems
    }
}

impl Default for PacelcAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cassandra_is_pa_el() {
        let analysis = PacelcAnalysis::new();
        let class = analysis.classify("Cassandra").unwrap();
        assert_eq!(class.partition, PartitionBehavior::PA);
        assert_eq!(class.normal, NormalBehavior::EL);
        assert_eq!(class.notation(), "PA/EL");
    }

    #[test]
    fn etcd_is_pc_ec() {
        let analysis = PacelcAnalysis::new();
        let class = analysis.classify("etcd").unwrap();
        assert_eq!(class.partition, PartitionBehavior::PC);
        assert_eq!(class.normal, NormalBehavior::EC);
        assert_eq!(class.notation(), "PC/EC");
    }

    #[test]
    fn zookeeper_is_pc_ec() {
        let analysis = PacelcAnalysis::new();
        let class = analysis.classify("ZooKeeper").unwrap();
        assert_eq!(class.partition, PartitionBehavior::PC);
        assert_eq!(class.normal, NormalBehavior::EC);
    }

    #[test]
    fn spanner_is_pc_ec() {
        let analysis = PacelcAnalysis::new();
        let class = analysis.classify("Spanner").unwrap();
        assert_eq!(class.partition, PartitionBehavior::PC);
        assert_eq!(class.normal, NormalBehavior::EC);
    }

    #[test]
    fn find_all_pa_el_systems() {
        let analysis = PacelcAnalysis::new();
        let systems = analysis.systems_in_category(
            PartitionBehavior::PA,
            NormalBehavior::EL,
        );
        let names: Vec<&str> = systems.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Cassandra"));
        assert!(names.contains(&"DynamoDB"));
    }

    #[test]
    fn register_custom_system() {
        let mut analysis = PacelcAnalysis::new();
        analysis.register(
            "CustomDB",
            PacelcClassification::new(PartitionBehavior::PC, NormalBehavior::EL),
            "Custom system that favors consistency during partition but latency normally.",
        );
        let class = analysis.classify("CustomDB").unwrap();
        assert_eq!(class.notation(), "PC/EL");
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        let analysis = PacelcAnalysis::new();
        assert!(analysis.classify("NonexistentDB").is_none());
    }
}
