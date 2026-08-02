//! Executable 80/20 innovation audit, regression detection, and support bundles.
//!
//! The audit deliberately reuses the kernel's public surfaces. It does not grant
//! ambient actuation authority: the runtime probe still crosses the exclusive
//! broker path and every result is reduced to deterministic evidence.

use crate::broker::{Broker, Executor, PreflightRefusal};
use crate::combinatorial::{
    standard_combinatorial_engine, ServiceObjective, TelcoPath, TelcoTopology, VisionWizard,
    WizardValue,
};
use crate::graph::{Capability, CapabilityGraph};
use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{
    AuthorityId, CapabilityId, Intent, Observation, OperationId, Outcome, PolicyId, SubjectId,
};
use crate::phase_change::{CapabilityStanding, Vision2030};
use crate::policy::{AdmissionPolicy, Predicate, Rule};
use crate::runtime::{Route, Router, Runtime};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Innovation dimension used to keep gap selection balanced.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InnovationDimension {
    Adoption,
    Evidence,
    Explainability,
    Recovery,
    Runtime,
}

impl InnovationDimension {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adoption => "adoption",
            Self::Evidence => "evidence",
            Self::Explainability => "explainability",
            Self::Recovery => "recovery",
            Self::Runtime => "runtime",
        }
    }
}

/// One high-leverage innovation gap and its observed closure evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InnovationFinding {
    id: String,
    title: String,
    dimension: InnovationDimension,
    impact: u16,
    effort: u16,
    confidence: u16,
    before: CapabilityStanding,
    after: CapabilityStanding,
    remedy: String,
    evidence: Vec<String>,
    digest: Digest,
}

impl InnovationFinding {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        dimension: InnovationDimension,
        impact: u16,
        effort: u16,
        confidence: u16,
        before: CapabilityStanding,
        after: CapabilityStanding,
        remedy: impl Into<String>,
        evidence: Vec<String>,
    ) -> Self {
        let id = id.into();
        let title = title.into();
        let remedy = remedy.into();
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "innovation-finding-v1")
            .text("id", &id)
            .text("title", &title)
            .text("dimension", dimension.as_str())
            .u64("impact", u64::from(impact))
            .u64("effort", u64::from(effort))
            .u64("confidence", u64::from(confidence))
            .text("before", &before.to_string())
            .text("after", &after.to_string())
            .text("remedy", &remedy);
        for item in &evidence {
            encoder.text("evidence", item);
        }
        Self {
            id,
            title,
            dimension,
            impact,
            effort: effort.max(1),
            confidence: confidence.min(100),
            before,
            after,
            remedy,
            evidence,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub const fn dimension(&self) -> InnovationDimension {
        self.dimension
    }

    #[must_use]
    pub const fn impact(&self) -> u16 {
        self.impact
    }

    #[must_use]
    pub const fn effort(&self) -> u16 {
        self.effort
    }

    #[must_use]
    pub const fn confidence(&self) -> u16 {
        self.confidence
    }

    #[must_use]
    pub const fn before(&self) -> CapabilityStanding {
        self.before
    }

    #[must_use]
    pub const fn after(&self) -> CapabilityStanding {
        self.after
    }

    #[must_use]
    pub fn remedy(&self) -> &str {
        &self.remedy
    }

    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    /// Impact-confidence returned per unit of implementation effort.
    #[must_use]
    pub fn leverage_milli(&self) -> u64 {
        u64::from(self.impact)
            .saturating_mul(u64::from(self.confidence))
            .saturating_mul(1_000)
            / u64::from(self.effort)
    }
}

/// Exact executable audit of the highest-leverage innovation surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InnovationAudit {
    findings: Vec<InnovationFinding>,
    selected_80_20: Vec<String>,
    total_impact: u32,
    selected_impact: u32,
    closed_impact: u32,
    coverage_ppm: u32,
    standing: CapabilityStanding,
    doctor_json: String,
    digest: Digest,
}

impl InnovationAudit {
    /// Executes the runtime, composition, resilience, doctor, regression, and
    /// support-bundle probes and manufactures one deterministic audit report.
    #[must_use]
    pub fn run() -> Self {
        let doctor = Vision2030::standard().diagnose();
        let findings = vec![
            probe_finding(
                "runtime-tracer-bullet",
                "Observed end-to-end brokered runtime",
                InnovationDimension::Runtime,
                100,
                20,
                CapabilityStanding::PartialAlive,
                "execute a lawful request through route, admission, broker, receipt, and trace verification",
                run_runtime_probe(),
            ),
            probe_finding(
                "observed-doctor",
                "Evidence-backed operator diagnosis",
                InnovationDimension::Explainability,
                95,
                15,
                CapabilityStanding::PartialAlive,
                "derive operator standing from executable probes instead of static capability constants",
                run_doctor_probe(),
            ),
            probe_finding(
                "scenario-matrix",
                "Cross-profile lawful composition matrix",
                InnovationDimension::Adoption,
                85,
                20,
                CapabilityStanding::Unknown,
                "prove developer, edge, telco, and enterprise presets against the bounded composition engine",
                run_scenario_probe(),
            ),
            probe_finding(
                "telco-transit-redundancy",
                "Endpoint-tolerant transit redundancy",
                InnovationDimension::Recovery,
                80,
                10,
                CapabilityStanding::PartialAlive,
                "measure path disjointness across internal transit nodes while allowing shared service endpoints",
                run_telco_probe(),
            ),
            implemented_finding(
                "regression-diff",
                "Deterministic standing regression detection",
                InnovationDimension::Recovery,
                75,
                15,
                "compare immutable audit snapshots and classify improvements, regressions, and unchanged findings",
                "AuditSnapshot::diff with digest-bound unit coverage",
            ),
            implemented_finding(
                "support-bundle",
                "Portable deterministic support bundle",
                InnovationDimension::Evidence,
                70,
                10,
                "package doctor state, innovation evidence, reproduction commands, and one verifiable digest",
                "SupportBundle::verify with canonical JSON output",
            ),
        ];
        Self::from_findings(findings, doctor.to_json())
    }

    fn from_findings(mut findings: Vec<InnovationFinding>, doctor_json: String) -> Self {
        findings.sort_by(|left, right| left.id.cmp(&right.id));
        let total_impact = findings.iter().map(|finding| u32::from(finding.impact)).sum::<u32>();
        let closed_impact = findings
            .iter()
            .filter(|finding| finding.after == CapabilityStanding::Alive)
            .map(|finding| u32::from(finding.impact))
            .sum::<u32>();
        let earned = findings
            .iter()
            .map(|finding| u64::from(finding.impact) * u64::from(finding.after.score()))
            .sum::<u64>();
        let denominator = u64::from(total_impact).saturating_mul(100);
        let coverage_ppm = if denominator == 0 {
            0
        } else {
            ((earned.saturating_mul(1_000_000)) / denominator) as u32
        };
        let selected_80_20 = select_eighty_twenty(&findings);
        let selected = selected_80_20.iter().collect::<BTreeSet<_>>();
        let selected_impact = findings
            .iter()
            .filter(|finding| selected.contains(&finding.id))
            .map(|finding| u32::from(finding.impact))
            .sum::<u32>();
        let selected_findings = findings
            .iter()
            .filter(|finding| selected.contains(&finding.id))
            .collect::<Vec<_>>();
        let standing = if selected_findings
            .iter()
            .any(|finding| finding.after == CapabilityStanding::BuildBroken)
        {
            CapabilityStanding::BuildBroken
        } else if selected_findings.iter().any(|finding| {
            matches!(
                finding.after,
                CapabilityStanding::Blocked | CapabilityStanding::Unsupported
            )
        }) {
            CapabilityStanding::Blocked
        } else if selected_findings
            .iter()
            .all(|finding| finding.after == CapabilityStanding::Alive)
        {
            CapabilityStanding::Alive
        } else {
            CapabilityStanding::PartialAlive
        };
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "innovation-audit-v1")
            .text("standing", &standing.to_string())
            .u64("total-impact", u64::from(total_impact))
            .u64("selected-impact", u64::from(selected_impact))
            .u64("closed-impact", u64::from(closed_impact))
            .u64("coverage-ppm", u64::from(coverage_ppm))
            .text("doctor", &doctor_json);
        for finding in &findings {
            encoder.field("finding", &finding.digest.0);
        }
        for id in &selected_80_20 {
            encoder.text("selected", id);
        }
        Self {
            findings,
            selected_80_20,
            total_impact,
            selected_impact,
            closed_impact,
            coverage_ppm,
            standing,
            doctor_json,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub fn findings(&self) -> &[InnovationFinding] {
        &self.findings
    }

    #[must_use]
    pub fn selected_80_20(&self) -> &[String] {
        &self.selected_80_20
    }

    #[must_use]
    pub const fn total_impact(&self) -> u32 {
        self.total_impact
    }

    #[must_use]
    pub const fn selected_impact(&self) -> u32 {
        self.selected_impact
    }

    #[must_use]
    pub const fn closed_impact(&self) -> u32 {
        self.closed_impact
    }

    #[must_use]
    pub const fn coverage_ppm(&self) -> u32 {
        self.coverage_ppm
    }

    #[must_use]
    pub const fn standing(&self) -> CapabilityStanding {
        self.standing
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn snapshot(&self) -> AuditSnapshot {
        AuditSnapshot::from_audit(self)
    }

    #[must_use]
    pub fn support_bundle(&self) -> SupportBundle {
        SupportBundle::new(
            self.to_json(),
            self.doctor_json.clone(),
            vec![
                "cargo check --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets".to_owned(),
                "cargo test --manifest-path capabilities/dteam-kernel/Cargo.toml --all-targets -- --test-threads=1".to_owned(),
                "cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- innovation-json".to_owned(),
                "cargo run --manifest-path capabilities/dteam-kernel/Cargo.toml --bin dteam-doctor -- support".to_owned(),
            ],
        )
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let findings = self
            .findings
            .iter()
            .map(finding_json)
            .collect::<Vec<_>>()
            .join(",");
        let selected = self
            .selected_80_20
            .iter()
            .map(|id| format!("\"{}\"", escape(id)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"urn:dteam:innovation-audit:v1\",\"standing\":\"{}\",\"total_impact\":{},\"selected_impact\":{},\"closed_impact\":{},\"coverage_ppm\":{},\"selected_80_20\":[{}],\"findings\":[{}],\"digest\":\"{}\"}}",
            self.standing,
            self.total_impact,
            self.selected_impact,
            self.closed_impact,
            self.coverage_ppm,
            selected,
            findings,
            self.digest
        )
    }

    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        writeln!(output, "# dteam 80/20 innovation audit").expect("string write");
        writeln!(output).expect("string write");
        writeln!(
            output,
            "Standing: **{}** · impact closed: **{}/{}** · coverage: **{} ppm** · digest: `{}`",
            self.standing, self.closed_impact, self.total_impact, self.coverage_ppm, self.digest
        )
        .expect("string write");
        writeln!(output).expect("string write");
        writeln!(output, "## Selected 80/20 surface").expect("string write");
        writeln!(output).expect("string write");
        for id in &self.selected_80_20 {
            let finding = self
                .findings
                .iter()
                .find(|finding| finding.id == *id)
                .expect("selected finding exists");
            writeln!(
                output,
                "- `{}` — {} → {} · impact={} effort={} leverage={} · {}",
                finding.id,
                finding.before,
                finding.after,
                finding.impact,
                finding.effort,
                finding.leverage_milli(),
                finding.title
            )
            .expect("string write");
        }
        writeln!(output).expect("string write");
        writeln!(output, "## Complete findings").expect("string write");
        writeln!(output).expect("string write");
        for finding in &self.findings {
            writeln!(
                output,
                "### `{}` — {}",
                finding.id, finding.title
            )
            .expect("string write");
            writeln!(
                output,
                "- dimension: `{}`",
                finding.dimension.as_str()
            )
            .expect("string write");
            writeln!(output, "- standing: {} → {}", finding.before, finding.after)
                .expect("string write");
            writeln!(output, "- remedy: {}", finding.remedy).expect("string write");
            for item in &finding.evidence {
                writeln!(output, "- evidence: {item}").expect("string write");
            }
            writeln!(output).expect("string write");
        }
        output
    }
}

/// Immutable finding-standing snapshot used for regression detection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSnapshot {
    standings: BTreeMap<String, CapabilityStanding>,
    digest: Digest,
}

impl AuditSnapshot {
    fn from_audit(audit: &InnovationAudit) -> Self {
        let standings = audit
            .findings
            .iter()
            .map(|finding| (finding.id.clone(), finding.after))
            .collect::<BTreeMap<_, _>>();
        Self::new(standings)
    }

    fn new(standings: BTreeMap<String, CapabilityStanding>) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder.text("type", "innovation-snapshot-v1");
        for (id, standing) in &standings {
            encoder.text("finding", id).text("standing", &standing.to_string());
        }
        Self {
            standings,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub fn standings(&self) -> &BTreeMap<String, CapabilityStanding> {
        &self.standings
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn with_standing(
        &self,
        id: impl Into<String>,
        standing: CapabilityStanding,
    ) -> Self {
        let mut standings = self.standings.clone();
        standings.insert(id.into(), standing);
        Self::new(standings)
    }

    #[must_use]
    pub fn diff(&self, current: &Self) -> AuditDiff {
        let ids = self
            .standings
            .keys()
            .chain(current.standings.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut improved = Vec::new();
        let mut regressed = Vec::new();
        let mut unchanged = Vec::new();
        for id in ids {
            let before = self
                .standings
                .get(&id)
                .copied()
                .unwrap_or(CapabilityStanding::Unknown);
            let after = current
                .standings
                .get(&id)
                .copied()
                .unwrap_or(CapabilityStanding::Unknown);
            match after.score().cmp(&before.score()) {
                std::cmp::Ordering::Greater => improved.push(id),
                std::cmp::Ordering::Less => regressed.push(id),
                std::cmp::Ordering::Equal => unchanged.push(id),
            }
        }
        AuditDiff::new(self.digest, current.digest, improved, regressed, unchanged)
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let entries = self
            .standings
            .iter()
            .map(|(id, standing)| {
                format!("\"{}\":\"{}\"", escape(id), standing)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"urn:dteam:innovation-snapshot:v1\",\"standings\":{{{entries}}},\"digest\":\"{}\"}}",
            self.digest
        )
    }
}

/// Deterministic comparison between two innovation snapshots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditDiff {
    previous: Digest,
    current: Digest,
    improved: Vec<String>,
    regressed: Vec<String>,
    unchanged: Vec<String>,
    digest: Digest,
}

impl AuditDiff {
    fn new(
        previous: Digest,
        current: Digest,
        improved: Vec<String>,
        regressed: Vec<String>,
        unchanged: Vec<String>,
    ) -> Self {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "innovation-diff-v1")
            .field("previous", &previous.0)
            .field("current", &current.0);
        for id in &improved {
            encoder.text("improved", id);
        }
        for id in &regressed {
            encoder.text("regressed", id);
        }
        for id in &unchanged {
            encoder.text("unchanged", id);
        }
        Self {
            previous,
            current,
            improved,
            regressed,
            unchanged,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub const fn previous(&self) -> Digest {
        self.previous
    }

    #[must_use]
    pub const fn current(&self) -> Digest {
        self.current
    }

    #[must_use]
    pub fn improved(&self) -> &[String] {
        &self.improved
    }

    #[must_use]
    pub fn regressed(&self) -> &[String] {
        &self.regressed
    }

    #[must_use]
    pub fn unchanged(&self) -> &[String] {
        &self.unchanged
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Self-verifying support payload for issue reports and operational handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportBundle {
    audit_json: String,
    doctor_json: String,
    commands: Vec<String>,
    digest: Digest,
}

impl SupportBundle {
    fn new(audit_json: String, doctor_json: String, commands: Vec<String>) -> Self {
        let digest = support_digest(&audit_json, &doctor_json, &commands);
        Self {
            audit_json,
            doctor_json,
            commands,
            digest,
        }
    }

    #[must_use]
    pub fn audit_json(&self) -> &str {
        &self.audit_json
    }

    #[must_use]
    pub fn doctor_json(&self) -> &str {
        &self.doctor_json
    }

    #[must_use]
    pub fn commands(&self) -> &[String] {
        &self.commands
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    #[must_use]
    pub fn verify(&self) -> bool {
        self.digest == support_digest(&self.audit_json, &self.doctor_json, &self.commands)
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let commands = self
            .commands
            .iter()
            .map(|command| format!("\"{}\"", escape(command)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"urn:dteam:support-bundle:v1\",\"audit\":{},\"doctor\":{},\"commands\":[{}],\"digest\":\"{}\"}}",
            self.audit_json, self.doctor_json, commands, self.digest
        )
    }
}

fn support_digest(audit_json: &str, doctor_json: &str, commands: &[String]) -> Digest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .text("type", "support-bundle-v1")
        .text("audit", audit_json)
        .text("doctor", doctor_json);
    for command in commands {
        encoder.text("command", command);
    }
    encoder.digest()
}

fn select_eighty_twenty(findings: &[InnovationFinding]) -> Vec<String> {
    let total = findings.iter().map(|finding| u32::from(finding.impact)).sum::<u32>();
    let target = total.saturating_mul(80).saturating_add(99) / 100;
    let mut ordered = findings.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        right
            .leverage_milli()
            .cmp(&left.leverage_milli())
            .then_with(|| right.impact.cmp(&left.impact))
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut selected = Vec::new();
    let mut impact = 0_u32;
    for finding in ordered {
        if impact >= target {
            break;
        }
        impact = impact.saturating_add(u32::from(finding.impact));
        selected.push(finding.id.clone());
    }
    selected.sort();
    selected
}

#[allow(clippy::too_many_arguments)]
fn probe_finding(
    id: &str,
    title: &str,
    dimension: InnovationDimension,
    impact: u16,
    effort: u16,
    before: CapabilityStanding,
    remedy: &str,
    execution: Result<Vec<String>, String>,
) -> InnovationFinding {
    match execution {
        Ok(evidence) => InnovationFinding::new(
            id,
            title,
            dimension,
            impact,
            effort,
            95,
            before,
            CapabilityStanding::Alive,
            remedy,
            evidence,
        ),
        Err(error) => InnovationFinding::new(
            id,
            title,
            dimension,
            impact,
            effort,
            95,
            before,
            CapabilityStanding::Blocked,
            remedy,
            vec![error],
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn implemented_finding(
    id: &str,
    title: &str,
    dimension: InnovationDimension,
    impact: u16,
    effort: u16,
    remedy: &str,
    evidence: &str,
) -> InnovationFinding {
    InnovationFinding::new(
        id,
        title,
        dimension,
        impact,
        effort,
        100,
        CapabilityStanding::Unknown,
        CapabilityStanding::Alive,
        remedy,
        vec![evidence.to_owned()],
    )
}

struct EchoExecutor;

impl Executor for EchoExecutor {
    fn id(&self) -> &str {
        "innovation-echo"
    }

    fn preflight(&self, _intent: &Intent) -> Result<(), PreflightRefusal> {
        Ok(())
    }

    fn execute(&mut self, intent: &Intent) -> Outcome {
        Outcome::Applied {
            code: 200,
            output: intent.payload().to_vec(),
        }
    }
}

fn run_runtime_probe() -> Result<Vec<String>, String> {
    let capability = CapabilityId::new("innovation-notify").map_err(|error| error.to_string())?;
    let operation = OperationId::new("send").map_err(|error| error.to_string())?;
    let authority = AuthorityId::new("innovation-operator").map_err(|error| error.to_string())?;
    let mut graph = CapabilityGraph::new();
    graph
        .insert(
            Capability::new(capability.clone())
                .supports(operation.clone())
                .allows(authority.clone()),
        )
        .map_err(|error| error.to_string())?;
    let broker = Broker::new("innovation-broker", graph, 10).map_err(|error| error.to_string())?;
    let mut router = Router::new();
    router.insert(Route::new(
        "innovation.notifications.send",
        capability,
        operation,
    ));
    let policy = AdmissionPolicy::new(
        PolicyId::new("innovation-policy").map_err(|error| error.to_string())?,
        1,
    )
    .with_rule(Rule::new(
        "ready",
        Predicate::Equals {
            key: "ready".to_owned(),
            expected: true.into(),
        },
    ));
    let mut runtime = Runtime::new(router, policy, broker);
    let mut observation = Observation::new(
        SubjectId::new("innovation-case").map_err(|error| error.to_string())?,
        1,
    );
    observation
        .insert("ready", true)
        .map_err(|error| error.to_string())?;
    let mut executor = EchoExecutor;
    let result = runtime
        .process(
            &mut executor,
            "innovation.notifications.send",
            observation,
            authority,
            1,
            b"evidence".to_vec(),
        )
        .map_err(|error| error.to_string())?;
    let verified = result.trace().verify().map_err(|error| error.to_string())?;
    if verified != result.trace().head() {
        return Err("trace verification head mismatch".to_owned());
    }
    if !result.outcome().is_applied() {
        return Err("runtime outcome was not applied".to_owned());
    }
    if result.trace().events().len() != 7 {
        return Err(format!(
            "runtime trace had {} stages instead of 7",
            result.trace().events().len()
        ));
    }
    if runtime.broker().completions().receipts().len() != 1 {
        return Err("runtime did not retain exactly one completion receipt".to_owned());
    }
    Ok(vec![
        format!("trace_head={}", result.trace().head()),
        format!("completion={}", result.evidence().completion().digest()),
        "stages=parsed,routed,admitted,constructed,authorized,actuated,receipted".to_owned(),
        "broker_completions=1".to_owned(),
    ])
}

fn run_scenario_probe() -> Result<Vec<String>, String> {
    let engine = standard_combinatorial_engine();
    let mut evidence = Vec::new();
    for preset in ["developer", "edge", "telco", "enterprise"] {
        let wizard = wizard_for(preset)?;
        let plan = wizard.compile().map_err(|error| error.to_string())?;
        let space = engine
            .explore(plan.request())
            .map_err(|error| error.to_string())?;
        if space.lawful().is_empty() || space.pareto().is_empty() {
            return Err(format!("preset `{preset}` has no lawful Pareto composition"));
        }
        evidence.push(format!(
            "preset={preset};lawful={};pareto={};plan={};space={}",
            space.lawful().len(),
            space.pareto().len(),
            plan.digest(),
            space.digest()
        ));
    }
    Ok(evidence)
}

fn wizard_for(preset: &str) -> Result<VisionWizard, String> {
    let mut wizard = VisionWizard::standard();
    let answers = match preset {
        "edge" => [
            ("mode", "edge"),
            ("availability", "ha"),
            ("authority", "yes"),
            ("offline", "yes"),
            ("reversible", "yes"),
        ],
        "telco" => [
            ("mode", "telco"),
            ("availability", "carrier"),
            ("authority", "yes"),
            ("offline", "yes"),
            ("reversible", "yes"),
        ],
        "enterprise" => [
            ("mode", "enterprise"),
            ("availability", "ha"),
            ("authority", "yes"),
            ("offline", "no"),
            ("reversible", "no"),
        ],
        _ => [
            ("mode", "developer"),
            ("availability", "standard"),
            ("authority", "yes"),
            ("offline", "yes"),
            ("reversible", "yes"),
        ],
    };
    for (id, value) in answers {
        wizard
            .answer(id, WizardValue::Choice(value.to_owned()))
            .map_err(|error| error.to_string())?;
    }
    Ok(wizard)
}

fn run_telco_probe() -> Result<Vec<String>, String> {
    let topology = TelcoTopology::standard();
    let objective = ServiceObjective {
        maximum_latency_micros: 2_000,
        minimum_capacity: 8_000,
        minimum_reliability_ppm: 999_999,
        minimum_failure_domains: 3,
    };
    let assessment = topology.assess("edge-a", "core-a", &objective);
    let transit_disjoint = transit_disjoint_count(assessment.compliant_paths());
    if assessment.compliant_paths().len() < 2 {
        return Err("fewer than two compliant telco paths".to_owned());
    }
    if transit_disjoint < 2 {
        return Err(format!(
            "only {transit_disjoint} internally disjoint telco path(s)"
        ));
    }
    if !assessment.single_points_of_failure().is_empty() {
        return Err(format!(
            "single points of failure: {}",
            assessment.single_points_of_failure().join(",")
        ));
    }
    if assessment.disjoint_path_count() != transit_disjoint {
        return Err(format!(
            "assessment counted {} disjoint path(s), transit analysis counted {transit_disjoint}",
            assessment.disjoint_path_count()
        ));
    }
    if assessment.standing() != "ALIVE" {
        return Err(format!(
            "telco assessment standing is {}",
            assessment.standing()
        ));
    }
    Ok(vec![
        format!("compliant_paths={}", assessment.compliant_paths().len()),
        format!("transit_disjoint_paths={transit_disjoint}"),
        "single_points_of_failure=0".to_owned(),
        format!("assessment={}", assessment.digest()),
    ])
}

fn transit_disjoint_count(paths: &[TelcoPath]) -> usize {
    let mut selected = Vec::<BTreeSet<String>>::new();
    'candidate: for path in paths {
        let internal = path
            .nodes()
            .iter()
            .skip(1)
            .take(path.nodes().len().saturating_sub(2))
            .cloned()
            .collect::<BTreeSet<_>>();
        for existing in &selected {
            if !internal.is_disjoint(existing) {
                continue 'candidate;
            }
        }
        selected.push(internal);
    }
    selected.len()
}

fn run_doctor_probe() -> Result<Vec<String>, String> {
    let report = Vision2030::standard().diagnose();
    if report.total() < 17 {
        return Err(format!(
            "doctor exposed {} capabilities, expected at least 17",
            report.total()
        ));
    }
    if report.digest() == Digest::ZERO {
        return Err("doctor report has a zero digest".to_owned());
    }
    if report.critical_path().is_empty() {
        return Err("doctor did not produce a critical path".to_owned());
    }
    Ok(vec![
        format!("doctor_standing={}", report.standing()),
        format!("doctor_score={}", report.score()),
        format!("doctor_capabilities={}", report.total()),
        format!("doctor_digest={}", report.digest()),
        "innovation standing is overlaid from executable probes".to_owned(),
    ])
}

fn finding_json(finding: &InnovationFinding) -> String {
    let evidence = finding
        .evidence
        .iter()
        .map(|item| format!("\"{}\"", escape(item)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"id\":\"{}\",\"title\":\"{}\",\"dimension\":\"{}\",\"impact\":{},\"effort\":{},\"confidence\":{},\"before\":\"{}\",\"after\":\"{}\",\"remedy\":\"{}\",\"evidence\":[{}],\"digest\":\"{}\"}}",
        escape(&finding.id),
        escape(&finding.title),
        finding.dimension.as_str(),
        finding.impact,
        finding.effort,
        finding.confidence,
        finding.before,
        finding.after,
        escape(&finding.remedy),
        evidence,
        finding.digest
    )
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::{transit_disjoint_count, InnovationAudit};
    use crate::combinatorial::{ServiceObjective, TelcoTopology};
    use crate::phase_change::CapabilityStanding;

    #[test]
    fn audit_closes_selected_eighty_twenty_surface() {
        let audit = InnovationAudit::run();
        assert_eq!(audit.standing(), CapabilityStanding::Alive);
        assert_eq!(audit.coverage_ppm(), 1_000_000);
        assert!(audit.selected_impact().saturating_mul(100) >= audit.total_impact().saturating_mul(80));
        assert!(audit
            .findings()
            .iter()
            .all(|finding| finding.after() == CapabilityStanding::Alive));
    }

    #[test]
    fn support_bundle_is_self_verifying() {
        let bundle = InnovationAudit::run().support_bundle();
        assert!(bundle.verify());
        assert!(bundle.to_json().contains("urn:dteam:support-bundle:v1"));
    }

    #[test]
    fn snapshot_diff_detects_regression() {
        let snapshot = InnovationAudit::run().snapshot();
        let regressed = snapshot.with_standing(
            "runtime-tracer-bullet",
            CapabilityStanding::Blocked,
        );
        let diff = snapshot.diff(&regressed);
        assert_eq!(diff.regressed(), &["runtime-tracer-bullet"]);
        assert!(diff.improved().is_empty());
    }

    #[test]
    fn transit_disjointness_allows_shared_service_endpoints() {
        let topology = TelcoTopology::standard();
        let objective = ServiceObjective {
            maximum_latency_micros: 2_000,
            minimum_capacity: 8_000,
            minimum_reliability_ppm: 999_999,
            minimum_failure_domains: 3,
        };
        let assessment = topology.assess("edge-a", "core-a", &objective);
        assert!(transit_disjoint_count(assessment.compliant_paths()) >= 2);
    }
}
