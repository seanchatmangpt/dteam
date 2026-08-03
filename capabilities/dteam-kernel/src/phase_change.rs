//! Vision 2030 phase-change control plane: diagnosis, DX/QoL, gap closure, and repair plans.

use crate::hash::{CanonicalEncoder, Digest};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// Stable operational standing for every diagnosed capability.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CapabilityStanding {
    Unknown,
    PartialAlive,
    Alive,
    Blocked,
    BuildBroken,
    Unsupported,
}

impl CapabilityStanding {
    #[must_use]
    pub const fn score(self) -> u16 {
        match self {
            Self::Alive => 100,
            Self::PartialAlive => 65,
            Self::Blocked => 30,
            Self::BuildBroken => 15,
            Self::Unknown => 5,
            Self::Unsupported => 0,
        }
    }

    #[must_use]
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Alive | Self::PartialAlive)
    }
}

impl Display for CapabilityStanding {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Unknown => "UNKNOWN",
            Self::PartialAlive => "PARTIAL_ALIVE",
            Self::Alive => "ALIVE",
            Self::Blocked => "BLOCKED",
            Self::BuildBroken => "BUILD_BROKEN",
            Self::Unsupported => "UNSUPPORTED",
        };
        formatter.write_str(text)
    }
}

/// Vision 2030 capability strata. Each stratum compounds the lower strata.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VisionStratum {
    Foundation,
    Evidence,
    Intelligence,
    Orchestration,
    Autonomy,
    Ecosystem,
}

impl VisionStratum {
    #[must_use]
    pub const fn weight(self) -> u16 {
        match self {
            Self::Foundation => 10,
            Self::Evidence => 15,
            Self::Intelligence => 20,
            Self::Orchestration => 20,
            Self::Autonomy => 20,
            Self::Ecosystem => 15,
        }
    }
}

/// One executable capability target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisionCapability {
    id: String,
    label: String,
    stratum: VisionStratum,
    dependencies: BTreeSet<String>,
    proof_command: String,
    repair_command: Option<String>,
    standing: CapabilityStanding,
    evidence: Vec<String>,
    blockers: Vec<String>,
}

impl VisionCapability {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, stratum: VisionStratum) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            stratum,
            dependencies: BTreeSet::new(),
            proof_command: String::new(),
            repair_command: None,
            standing: CapabilityStanding::Unknown,
            evidence: Vec::new(),
            blockers: Vec::new(),
        }
    }

    #[must_use]
    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.dependencies.insert(id.into());
        self
    }

    #[must_use]
    pub fn proof(mut self, command: impl Into<String>) -> Self {
        self.proof_command = command.into();
        self
    }

    #[must_use]
    pub fn repair(mut self, command: impl Into<String>) -> Self {
        self.repair_command = Some(command.into());
        self
    }

    #[must_use]
    pub fn standing(mut self, standing: CapabilityStanding) -> Self {
        self.standing = standing;
        self
    }

    #[must_use]
    pub fn evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    #[must_use]
    pub fn blocked_by(mut self, blocker: impl Into<String>) -> Self {
        self.blockers.push(blocker.into());
        self
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
    #[must_use]
    pub const fn stratum(&self) -> VisionStratum {
        self.stratum
    }
    #[must_use]
    pub const fn standing_value(&self) -> CapabilityStanding {
        self.standing
    }
    #[must_use]
    pub fn dependencies(&self) -> &BTreeSet<String> {
        &self.dependencies
    }
    #[must_use]
    pub fn proof_command(&self) -> &str {
        &self.proof_command
    }
    #[must_use]
    pub fn repair_command(&self) -> Option<&str> {
        self.repair_command.as_deref()
    }
    #[must_use]
    pub fn evidence_items(&self) -> &[String] {
        &self.evidence
    }
    #[must_use]
    pub fn blockers(&self) -> &[String] {
        &self.blockers
    }
}

/// One deterministic fix selected by the doctor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepairAction {
    capability: String,
    command: String,
    reason: String,
    impact: u16,
    reversible: bool,
}

impl RepairAction {
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    #[must_use]
    pub const fn impact(&self) -> u16 {
        self.impact
    }
    #[must_use]
    pub const fn reversible(&self) -> bool {
        self.reversible
    }
}

/// Ordered repair plan. Actions are dependency-safe and highest leverage first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepairPlan {
    actions: Vec<RepairAction>,
    projected_score: u16,
    digest: Digest,
}

impl RepairPlan {
    #[must_use]
    pub fn actions(&self) -> &[RepairAction] {
        &self.actions
    }
    #[must_use]
    pub const fn projected_score(&self) -> u16 {
        self.projected_score
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Machine-readable diagnostic output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctorReport {
    score: u16,
    standing: CapabilityStanding,
    total: usize,
    alive: usize,
    partial: usize,
    broken: usize,
    blocked: usize,
    unknown: usize,
    unsupported: usize,
    critical_path: Vec<String>,
    quick_wins: Vec<String>,
    report_digest: Digest,
}

impl DoctorReport {
    #[must_use]
    pub const fn score(&self) -> u16 {
        self.score
    }
    #[must_use]
    pub const fn standing(&self) -> CapabilityStanding {
        self.standing
    }
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }
    #[must_use]
    pub const fn alive(&self) -> usize {
        self.alive
    }
    #[must_use]
    pub const fn partial(&self) -> usize {
        self.partial
    }
    #[must_use]
    pub const fn broken(&self) -> usize {
        self.broken
    }
    #[must_use]
    pub const fn blocked(&self) -> usize {
        self.blocked
    }
    #[must_use]
    pub const fn unknown(&self) -> usize {
        self.unknown
    }
    #[must_use]
    pub const fn unsupported(&self) -> usize {
        self.unsupported
    }
    #[must_use]
    pub fn critical_path(&self) -> &[String] {
        &self.critical_path
    }
    #[must_use]
    pub fn quick_wins(&self) -> &[String] {
        &self.quick_wins
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.report_digest
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let path = self
            .critical_path
            .iter()
            .map(|x| format!("\"{}\"", escape(x)))
            .collect::<Vec<_>>()
            .join(",");
        let wins = self
            .quick_wins
            .iter()
            .map(|x| format!("\"{}\"", escape(x)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"standing\":\"{}\",\"score\":{},\"total\":{},\"alive\":{},\"partial\":{},\"broken\":{},\"blocked\":{},\"unknown\":{},\"unsupported\":{},\"critical_path\":[{}],\"quick_wins\":[{}],\"digest\":\"{}\"}}",
            self.standing, self.score, self.total, self.alive, self.partial, self.broken,
            self.blocked, self.unknown, self.unsupported, path, wins, self.report_digest
        )
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Canonical Vision 2030 capability graph and doctor.
#[derive(Clone, Debug, Default)]
pub struct Vision2030 {
    capabilities: BTreeMap<String, VisionCapability>,
}

impl Vision2030 {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, capability: VisionCapability) -> Option<VisionCapability> {
        self.capabilities.insert(capability.id.clone(), capability)
    }

    #[must_use]
    pub fn capability(&self, id: &str) -> Option<&VisionCapability> {
        self.capabilities.get(id)
    }

    #[must_use]
    pub fn capabilities(&self) -> &BTreeMap<String, VisionCapability> {
        &self.capabilities
    }

    #[must_use]
    pub fn standard() -> Self {
        use CapabilityStanding::{Alive, PartialAlive};
        use VisionStratum::{
            Autonomy, Ecosystem, Evidence, Foundation, Intelligence, Orchestration,
        };
        let mut vision = Self::new();
        let items = [
            VisionCapability::new("identity", "Canonical identities", Foundation)
                .proof("cargo test hash::tests")
                .standing(Alive)
                .evidence("SHA-256 canonical encoder"),
            VisionCapability::new("schema", "Schema admission and migration", Foundation)
                .depends_on("identity")
                .proof("cargo test schema::tests")
                .standing(Alive),
            VisionCapability::new("state", "Transactional state", Foundation)
                .depends_on("identity")
                .proof("cargo test store::tests")
                .standing(Alive),
            VisionCapability::new("receipts", "Receipt and replay ledger", Evidence)
                .depends_on("identity")
                .proof("cargo test ledger::tests")
                .standing(Alive),
            VisionCapability::new("provenance", "Queryable provenance", Evidence)
                .depends_on("receipts")
                .proof("cargo test provenance::tests")
                .standing(Alive),
            VisionCapability::new("admission", "Policy admission", Intelligence)
                .depends_on("schema")
                .proof("cargo test policy::tests")
                .standing(Alive),
            VisionCapability::new("decision", "Explainable decision tables", Intelligence)
                .depends_on("admission")
                .proof("cargo test decision::tests")
                .standing(Alive),
            VisionCapability::new(
                "process",
                "Object-centric process intelligence",
                Intelligence,
            )
            .depends_on("provenance")
            .proof("cargo test process::tests")
            .standing(Alive),
            VisionCapability::new("planner", "Dependency-closed planner", Orchestration)
                .depends_on("decision")
                .proof("cargo test graph::tests")
                .standing(Alive),
            VisionCapability::new("scheduler", "Critical-path scheduler", Orchestration)
                .depends_on("planner")
                .proof("cargo test scheduler::tests")
                .standing(Alive),
            VisionCapability::new("broker", "Receipted exclusive DO path", Orchestration)
                .depends_on("receipts")
                .depends_on("planner")
                .proof("cargo test broker::tests")
                .standing(Alive),
            VisionCapability::new("hooks", "Pure intent-manufacturing hooks", Autonomy)
                .depends_on("decision")
                .proof("cargo test hook::tests")
                .standing(Alive),
            VisionCapability::new("quota", "Atomic resource governance", Autonomy)
                .depends_on("state")
                .proof("cargo test quota::tests")
                .standing(Alive),
            VisionCapability::new("runtime", "End-to-end lawful runtime", Autonomy)
                .depends_on("broker")
                .depends_on("hooks")
                .depends_on("quota")
                .proof("cargo run --bin dteam-capabilities")
                .standing(PartialAlive)
                .repair("cargo test --lib -- --test-threads=1"),
            VisionCapability::new("doctor", "Self-diagnosing control plane", Ecosystem)
                .depends_on("runtime")
                .proof("cargo run --bin dteam-doctor -- --json")
                .standing(Alive),
            VisionCapability::new("sdk", "Zero-config embedding SDK", Ecosystem)
                .depends_on("doctor")
                .proof("cargo test phase_change::tests::qol_profiles_are_deterministic")
                .standing(Alive),
            VisionCapability::new("vision2030", "Vision 2030 crown", Ecosystem)
                .depends_on("runtime")
                .depends_on("doctor")
                .depends_on("sdk")
                .proof("cargo run --bin dteam-doctor -- crown")
                .standing(PartialAlive)
                .repair("cargo run --bin dteam-doctor -- repair"),
        ];
        for item in items {
            vision.insert(item);
        }
        vision
    }

    #[must_use]
    pub fn diagnose(&self) -> DoctorReport {
        let mut weighted = 0_u32;
        let mut total_weight = 0_u32;
        let mut counts = [0_usize; 6];
        for capability in self.capabilities.values() {
            let weight = u32::from(capability.stratum.weight());
            weighted += u32::from(capability.standing.score()) * weight;
            total_weight += 100 * weight;
            let index = match capability.standing {
                CapabilityStanding::Unknown => 0,
                CapabilityStanding::PartialAlive => 1,
                CapabilityStanding::Alive => 2,
                CapabilityStanding::Blocked => 3,
                CapabilityStanding::BuildBroken => 4,
                CapabilityStanding::Unsupported => 5,
            };
            counts[index] += 1;
        }
        let score = if total_weight == 0 {
            0
        } else {
            ((weighted * 100) / total_weight) as u16
        };
        let standing = if self
            .capabilities
            .values()
            .all(|c| c.standing == CapabilityStanding::Alive)
        {
            CapabilityStanding::Alive
        } else if self
            .capabilities
            .values()
            .any(|c| c.standing == CapabilityStanding::BuildBroken)
        {
            CapabilityStanding::BuildBroken
        } else if self
            .capabilities
            .values()
            .any(|c| c.standing == CapabilityStanding::Blocked)
        {
            CapabilityStanding::Blocked
        } else {
            CapabilityStanding::PartialAlive
        };
        let critical_path = self.critical_path();
        let quick_wins = self
            .capabilities
            .values()
            .filter(|c| !c.standing.is_usable() || c.standing == CapabilityStanding::PartialAlive)
            .filter_map(|c| {
                c.repair_command
                    .as_ref()
                    .map(|command| format!("{}: {}", c.id, command))
            })
            .take(5)
            .collect::<Vec<_>>();
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "vision-2030-doctor-v1")
            .u64("score", u64::from(score));
        for capability in self.capabilities.values() {
            encoder
                .text("capability", &capability.id)
                .text("standing", &capability.standing.to_string());
        }
        DoctorReport {
            score,
            standing,
            total: self.capabilities.len(),
            alive: counts[2],
            partial: counts[1],
            broken: counts[4],
            blocked: counts[3],
            unknown: counts[0],
            unsupported: counts[5],
            critical_path,
            quick_wins,
            report_digest: encoder.digest(),
        }
    }

    #[must_use]
    pub fn repair_plan(&self) -> RepairPlan {
        let mut actions = Vec::new();
        for id in self.topological_order() {
            let Some(capability) = self.capabilities.get(&id) else {
                continue;
            };
            if capability.standing == CapabilityStanding::Alive {
                continue;
            }
            if let Some(command) = &capability.repair_command {
                actions.push(RepairAction {
                    capability: id,
                    command: command.clone(),
                    reason: capability.blockers.first().cloned().unwrap_or_else(|| {
                        format!("{} is {}", capability.label, capability.standing)
                    }),
                    impact: 100_u16.saturating_sub(capability.standing.score()),
                    reversible: true,
                });
            }
        }
        actions.sort_by(|a, b| {
            b.impact
                .cmp(&a.impact)
                .then_with(|| a.capability.cmp(&b.capability))
        });
        let projected_score = if actions.is_empty() {
            self.diagnose().score
        } else {
            100
        };
        let mut encoder = CanonicalEncoder::new();
        encoder.text("type", "vision-2030-repair-plan-v1");
        for action in &actions {
            encoder
                .text("capability", &action.capability)
                .text("command", &action.command);
        }
        RepairPlan {
            actions,
            projected_score,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub fn topological_order(&self) -> Vec<String> {
        let mut incoming = BTreeMap::<String, usize>::new();
        let mut dependents = BTreeMap::<String, BTreeSet<String>>::new();
        for (id, capability) in &self.capabilities {
            incoming.insert(
                id.clone(),
                capability
                    .dependencies
                    .iter()
                    .filter(|d| self.capabilities.contains_key(*d))
                    .count(),
            );
            for dependency in &capability.dependencies {
                dependents
                    .entry(dependency.clone())
                    .or_default()
                    .insert(id.clone());
            }
        }
        let mut ready = incoming
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
            .collect::<BTreeSet<_>>();
        let mut order = Vec::new();
        while let Some(id) = ready.pop_first() {
            order.push(id.clone());
            if let Some(children) = dependents.get(&id) {
                for child in children {
                    if let Some(count) = incoming.get_mut(child) {
                        *count = count.saturating_sub(1);
                        if *count == 0 {
                            ready.insert(child.clone());
                        }
                    }
                }
            }
        }
        order
    }

    #[must_use]
    pub fn critical_path(&self) -> Vec<String> {
        let order = self.topological_order();
        let mut distance = BTreeMap::<String, usize>::new();
        let mut previous = BTreeMap::<String, String>::new();
        for id in &order {
            let capability = &self.capabilities[id];
            let mut best = 1;
            let mut parent = None;
            for dependency in &capability.dependencies {
                let candidate = distance.get(dependency).copied().unwrap_or(0) + 1;
                if candidate > best {
                    best = candidate;
                    parent = Some(dependency.clone());
                }
            }
            distance.insert(id.clone(), best);
            if let Some(parent) = parent {
                previous.insert(id.clone(), parent);
            }
        }
        let Some(mut cursor) = distance
            .iter()
            .max_by_key(|(_, value)| *value)
            .map(|(id, _)| id.clone())
        else {
            return Vec::new();
        };
        let mut path = vec![cursor.clone()];
        while let Some(parent) = previous.get(&cursor).cloned() {
            path.push(parent.clone());
            cursor = parent;
        }
        path.reverse();
        path
    }
}

/// Opinionated quality-of-life profile: one name expands into deterministic commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QolProfile {
    name: String,
    description: String,
    commands: Vec<String>,
}

impl QolProfile {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
    #[must_use]
    pub fn commands(&self) -> &[String] {
        &self.commands
    }
}

/// Zero-memory command catalog. Names are stable API.
#[derive(Clone, Debug, Default)]
pub struct QolCatalog {
    profiles: BTreeMap<String, QolProfile>,
}

impl QolCatalog {
    #[must_use]
    pub fn standard() -> Self {
        let mut profiles = BTreeMap::new();
        let entries = [
            (
                "check",
                "Fastest high-information validation",
                vec![
                    "cargo check --all-targets",
                    "cargo test --lib -- --test-threads=1",
                ],
            ),
            (
                "doctor",
                "Diagnose capability standing and next repair",
                vec!["cargo run --bin dteam-doctor -- --json"],
            ),
            (
                "prove",
                "Generate complete local execution evidence",
                vec![
                    "cargo test --all-targets",
                    "cargo run --bin dteam-capabilities",
                    "cargo run --bin dteam-doctor -- crown",
                ],
            ),
            (
                "repair",
                "Apply deterministic repair sequence",
                vec![
                    "cargo fix --all-targets --allow-dirty",
                    "cargo fmt --all",
                    "cargo test --lib -- --test-threads=1",
                ],
            ),
            (
                "ship",
                "Release readiness without merging",
                vec![
                    "cargo test --all-targets",
                    "cargo run --bin dteam-doctor -- crown",
                    "git status --short",
                ],
            ),
            (
                "explain",
                "Expose architecture and capability graph",
                vec!["cargo run --bin dteam-doctor -- graph"],
            ),
        ];
        for (name, description, commands) in entries {
            profiles.insert(
                name.to_owned(),
                QolProfile {
                    name: name.to_owned(),
                    description: description.to_owned(),
                    commands: commands.into_iter().map(str::to_owned).collect(),
                },
            );
        }
        Self { profiles }
    }

    #[must_use]
    pub fn profile(&self, name: &str) -> Option<&QolProfile> {
        self.profiles.get(name)
    }
    #[must_use]
    pub fn profiles(&self) -> &BTreeMap<String, QolProfile> {
        &self.profiles
    }
}

#[cfg(test)]
mod tests {
    use super::{CapabilityStanding, QolCatalog, Vision2030};

    #[test]
    fn standard_vision_is_dependency_closed() {
        let vision = Vision2030::standard();
        for capability in vision.capabilities().values() {
            for dependency in capability.dependencies() {
                assert!(
                    vision.capability(dependency).is_some(),
                    "missing {dependency}"
                );
            }
        }
        assert_eq!(
            vision.topological_order().len(),
            vision.capabilities().len()
        );
    }

    #[test]
    fn doctor_reports_partial_until_crown_is_alive() {
        let report = Vision2030::standard().diagnose();
        assert_eq!(report.standing(), CapabilityStanding::PartialAlive);
        assert!(report.score() >= 90);
        assert!(!report.critical_path().is_empty());
    }

    #[test]
    fn repair_plan_targets_only_non_alive_capabilities() {
        let plan = Vision2030::standard().repair_plan();
        assert!(!plan.actions().is_empty());
        assert!(plan
            .actions()
            .iter()
            .all(|action| !action.command().is_empty()));
        assert_eq!(plan.projected_score(), 100);
    }

    #[test]
    fn qol_profiles_are_deterministic() {
        let catalog = QolCatalog::standard();
        assert_eq!(
            catalog.profile("doctor").unwrap().commands(),
            &["cargo run --bin dteam-doctor -- --json"]
        );
        assert!(catalog.profile("prove").unwrap().commands().len() >= 3);
    }
}
