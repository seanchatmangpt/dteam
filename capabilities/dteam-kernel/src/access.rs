//! Explainable role-based access control with inheritance, delegation, scope, and replay.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{FactValue, Observation};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{Display, Formatter};

macro_rules! access_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, AccessError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(AccessError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

access_id!(AccessPrincipalId);
access_id!(RoleId);
access_id!(PermissionId);
access_id!(AccessResourceId);
access_id!(GrantId);
access_id!(DelegationId);

/// Hierarchical resource scope. A scope matches itself and descendants.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResourceScope {
    prefix: String,
}

impl ResourceScope {
    pub fn new(value: impl Into<String>) -> Result<Self, AccessError> {
        let mut value = value.into();
        if value.trim().is_empty() {
            return Err(AccessError::EmptyScope);
        }
        while value.ends_with('/') && value.len() > 1 {
            value.pop();
        }
        if !value.starts_with('/') {
            return Err(AccessError::RelativeScope(value));
        }
        if value.contains("//") || value.split('/').any(|segment| segment == "..") {
            return Err(AccessError::InvalidScope(value));
        }
        Ok(Self { prefix: value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.prefix
    }

    #[must_use]
    pub fn matches(&self, resource: &AccessResourceId) -> bool {
        resource.as_str() == self.prefix
            || resource
                .as_str()
                .strip_prefix(&self.prefix)
                .is_some_and(|suffix| self.prefix == "/" || suffix.starts_with('/'))
    }

    #[must_use]
    pub fn specificity(&self) -> usize {
        self.prefix.split('/').filter(|segment| !segment.is_empty()).count()
    }
}

/// Context constraint attached to a grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessConstraint {
    FactEquals { key: String, expected: FactValue },
    FactPresent { key: String },
    U64AtMost { key: String, maximum: u64 },
    U64AtLeast { key: String, minimum: u64 },
    TextSetContains { key: String, member: String },
}

impl AccessConstraint {
    fn evaluate(&self, context: &Observation) -> Result<(), String> {
        match self {
            Self::FactEquals { key, expected } => match context.fact(key) {
                Some(actual) if actual == expected => Ok(()),
                Some(actual) => Err(format!(
                    "fact `{key}` was {actual:?}, expected {expected:?}"
                )),
                None => Err(format!("fact `{key}` is absent")),
            },
            Self::FactPresent { key } => context
                .fact(key)
                .map(|_| ())
                .ok_or_else(|| format!("fact `{key}` is absent")),
            Self::U64AtMost { key, maximum } => match context.fact(key) {
                Some(FactValue::U64(actual)) if actual <= maximum => Ok(()),
                Some(FactValue::U64(actual)) => {
                    Err(format!("fact `{key}` was {actual}, maximum is {maximum}"))
                }
                Some(actual) => Err(format!("fact `{key}` was {actual:?}, expected u64")),
                None => Err(format!("fact `{key}` is absent")),
            },
            Self::U64AtLeast { key, minimum } => match context.fact(key) {
                Some(FactValue::U64(actual)) if actual >= minimum => Ok(()),
                Some(FactValue::U64(actual)) => {
                    Err(format!("fact `{key}` was {actual}, minimum is {minimum}"))
                }
                Some(actual) => Err(format!("fact `{key}` was {actual:?}, expected u64")),
                None => Err(format!("fact `{key}` is absent")),
            },
            Self::TextSetContains { key, member } => match context.fact(key) {
                Some(FactValue::TextSet(values)) if values.contains(member) => Ok(()),
                Some(FactValue::TextSet(_)) => {
                    Err(format!("fact `{key}` does not contain `{member}`"))
                }
                Some(actual) => Err(format!(
                    "fact `{key}` was {actual:?}, expected text set"
                )),
                None => Err(format!("fact `{key}` is absent")),
            },
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::FactEquals { key, expected } => {
                encoder
                    .text("constraint", "fact-equals")
                    .text("key", key);
                expected.encode(encoder, "expected-type");
            }
            Self::FactPresent { key } => {
                encoder
                    .text("constraint", "fact-present")
                    .text("key", key);
            }
            Self::U64AtMost { key, maximum } => {
                encoder
                    .text("constraint", "u64-at-most")
                    .text("key", key)
                    .u64("maximum", *maximum);
            }
            Self::U64AtLeast { key, minimum } => {
                encoder
                    .text("constraint", "u64-at-least")
                    .text("key", key)
                    .u64("minimum", *minimum);
            }
            Self::TextSetContains { key, member } => {
                encoder
                    .text("constraint", "text-set-contains")
                    .text("key", key)
                    .text("member", member);
            }
        }
    }
}

/// Grant effect. Deny overrides allow at equal or greater specificity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AccessEffect {
    Allow,
    Deny,
}

impl AccessEffect {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

/// Permission grant attached to a role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessGrant {
    id: GrantId,
    role: RoleId,
    permission: PermissionId,
    scope: ResourceScope,
    effect: AccessEffect,
    priority: i32,
    constraints: Vec<AccessConstraint>,
}

impl AccessGrant {
    #[must_use]
    pub fn new(
        id: GrantId,
        role: RoleId,
        permission: PermissionId,
        scope: ResourceScope,
        effect: AccessEffect,
    ) -> Self {
        Self {
            id,
            role,
            permission,
            scope,
            effect,
            priority: 0,
            constraints: Vec::new(),
        }
    }

    #[must_use]
    pub const fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn constrained_by(mut self, constraint: AccessConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    #[must_use]
    pub const fn id(&self) -> &GrantId {
        &self.id
    }

    #[must_use]
    pub const fn role(&self) -> &RoleId {
        &self.role
    }

    #[must_use]
    pub const fn permission(&self) -> &PermissionId {
        &self.permission
    }

    #[must_use]
    pub const fn scope(&self) -> &ResourceScope {
        &self.scope
    }

    #[must_use]
    pub const fn effect(&self) -> AccessEffect {
        self.effect
    }

    #[must_use]
    pub const fn priority_value(&self) -> i32 {
        self.priority
    }

    #[must_use]
    pub fn constraints(&self) -> &[AccessConstraint] {
        &self.constraints
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "access-grant-v1")
            .text("id", self.id.as_str())
            .text("role", self.role.as_str())
            .text("permission", self.permission.as_str())
            .text("scope", self.scope.as_str())
            .text("effect", self.effect.as_str())
            .i64("priority", i64::from(self.priority))
            .u64("constraint-count", self.constraints.len() as u64);
        for constraint in &self.constraints {
            constraint.encode(&mut encoder);
        }
        encoder.digest()
    }
}

/// Direct principal-to-role assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssignment {
    principal: AccessPrincipalId,
    role: RoleId,
    valid_from: u64,
    valid_until: Option<u64>,
}

impl RoleAssignment {
    pub fn new(
        principal: AccessPrincipalId,
        role: RoleId,
        valid_from: u64,
        valid_until: Option<u64>,
    ) -> Result<Self, AccessError> {
        if valid_until.is_some_and(|until| until <= valid_from) {
            return Err(AccessError::InvalidValidityWindow {
                valid_from,
                valid_until,
            });
        }
        Ok(Self {
            principal,
            role,
            valid_from,
            valid_until,
        })
    }

    #[must_use]
    pub const fn principal(&self) -> &AccessPrincipalId {
        &self.principal
    }

    #[must_use]
    pub const fn role(&self) -> &RoleId {
        &self.role
    }

    #[must_use]
    pub const fn active_at(&self, logical_time: u64) -> bool {
        logical_time >= self.valid_from
            && match self.valid_until {
                Some(until) => logical_time < until,
                None => true,
            }
    }
}

/// Time-bounded role delegation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegation {
    id: DelegationId,
    delegator: AccessPrincipalId,
    delegate: AccessPrincipalId,
    role: RoleId,
    scope: ResourceScope,
    valid_from: u64,
    valid_until: u64,
}

impl Delegation {
    pub fn new(
        id: DelegationId,
        delegator: AccessPrincipalId,
        delegate: AccessPrincipalId,
        role: RoleId,
        scope: ResourceScope,
        valid_from: u64,
        valid_until: u64,
    ) -> Result<Self, AccessError> {
        if delegator == delegate {
            return Err(AccessError::SelfDelegation(delegator));
        }
        if valid_until <= valid_from {
            return Err(AccessError::InvalidValidityWindow {
                valid_from,
                valid_until: Some(valid_until),
            });
        }
        Ok(Self {
            id,
            delegator,
            delegate,
            role,
            scope,
            valid_from,
            valid_until,
        })
    }

    #[must_use]
    pub const fn id(&self) -> &DelegationId {
        &self.id
    }

    #[must_use]
    pub const fn delegator(&self) -> &AccessPrincipalId {
        &self.delegator
    }

    #[must_use]
    pub const fn delegate(&self) -> &AccessPrincipalId {
        &self.delegate
    }

    #[must_use]
    pub const fn role(&self) -> &RoleId {
        &self.role
    }

    #[must_use]
    pub const fn scope(&self) -> &ResourceScope {
        &self.scope
    }

    #[must_use]
    pub const fn active_at(&self, logical_time: u64) -> bool {
        logical_time >= self.valid_from && logical_time < self.valid_until
    }
}

/// Separation-of-duty rule: no principal may activate more than `maximum` roles from the set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparationRule {
    id: String,
    roles: BTreeSet<RoleId>,
    maximum: usize,
}

impl SeparationRule {
    pub fn new(
        id: impl Into<String>,
        roles: BTreeSet<RoleId>,
        maximum: usize,
    ) -> Result<Self, AccessError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(AccessError::EmptySeparationId);
        }
        if roles.len() < 2 || maximum == 0 || maximum >= roles.len() {
            return Err(AccessError::InvalidSeparationRule {
                roles: roles.len(),
                maximum,
            });
        }
        Ok(Self { id, roles, maximum })
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn roles(&self) -> &BTreeSet<RoleId> {
        &self.roles
    }

    #[must_use]
    pub const fn maximum(&self) -> usize {
        self.maximum
    }
}

/// One candidate grant evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantEvaluation {
    grant: GrantId,
    role: RoleId,
    role_path: Vec<RoleId>,
    scope_matched: bool,
    constraints_matched: bool,
    failures: Vec<String>,
    digest: Digest,
}

impl GrantEvaluation {
    #[must_use]
    pub const fn grant(&self) -> &GrantId {
        &self.grant
    }

    #[must_use]
    pub const fn role(&self) -> &RoleId {
        &self.role
    }

    #[must_use]
    pub fn role_path(&self) -> &[RoleId] {
        &self.role_path
    }

    #[must_use]
    pub const fn scope_matched(&self) -> bool {
        self.scope_matched
    }

    #[must_use]
    pub const fn constraints_matched(&self) -> bool {
        self.constraints_matched
    }

    #[must_use]
    pub fn failures(&self) -> &[String] {
        &self.failures
    }
}

/// Explainable authorization decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessDecision {
    Allowed {
        grant: GrantId,
        role_path: Vec<RoleId>,
        evaluations: Vec<GrantEvaluation>,
        digest: Digest,
    },
    Denied {
        code: &'static str,
        grant: Option<GrantId>,
        evaluations: Vec<GrantEvaluation>,
        digest: Digest,
    },
}

impl AccessDecision {
    #[must_use]
    pub const fn allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    #[must_use]
    pub fn evaluations(&self) -> &[GrantEvaluation] {
        match self {
            Self::Allowed { evaluations, .. } | Self::Denied { evaluations, .. } => evaluations,
        }
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        match self {
            Self::Allowed { digest, .. } | Self::Denied { digest, .. } => *digest,
        }
    }
}

/// Policy construction or validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessError {
    EmptyIdentifier(&'static str),
    EmptyScope,
    RelativeScope(String),
    InvalidScope(String),
    DuplicateRole(RoleId),
    UnknownRole(RoleId),
    DuplicateGrant(GrantId),
    DuplicateDelegation(DelegationId),
    RoleCycle(Vec<RoleId>),
    InvalidValidityWindow {
        valid_from: u64,
        valid_until: Option<u64>,
    },
    SelfDelegation(AccessPrincipalId),
    DelegatorLacksRole {
        delegator: AccessPrincipalId,
        role: RoleId,
    },
    DelegationScopeExceeded {
        delegation: DelegationId,
        allowed: ResourceScope,
        requested: ResourceScope,
    },
    EmptySeparationId,
    InvalidSeparationRule { roles: usize, maximum: usize },
    SeparationViolation {
        principal: AccessPrincipalId,
        rule: String,
        active_roles: Vec<RoleId>,
    },
}

impl Display for AccessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(formatter, "{kind} must not be empty"),
            Self::EmptyScope => formatter.write_str("resource scope must not be empty"),
            Self::RelativeScope(scope) => write!(formatter, "scope `{scope}` must be absolute"),
            Self::InvalidScope(scope) => write!(formatter, "scope `{scope}` is invalid"),
            Self::DuplicateRole(role) => write!(formatter, "duplicate role `{role}`"),
            Self::UnknownRole(role) => write!(formatter, "unknown role `{role}`"),
            Self::DuplicateGrant(grant) => write!(formatter, "duplicate grant `{grant}`"),
            Self::DuplicateDelegation(id) => write!(formatter, "duplicate delegation `{id}`"),
            Self::RoleCycle(path) => {
                formatter.write_str("role inheritance cycle:")?;
                for role in path {
                    write!(formatter, " {role}")?;
                }
                Ok(())
            }
            Self::InvalidValidityWindow {
                valid_from,
                valid_until,
            } => write!(
                formatter,
                "validity window starts at {valid_from} and ends at {valid_until:?}"
            ),
            Self::SelfDelegation(principal) => {
                write!(formatter, "principal `{principal}` cannot delegate to itself")
            }
            Self::DelegatorLacksRole { delegator, role } => write!(
                formatter,
                "delegator `{delegator}` does not hold role `{role}`"
            ),
            Self::DelegationScopeExceeded {
                delegation,
                allowed,
                requested,
            } => write!(
                formatter,
                "delegation `{delegation}` scope `{requested:?}` exceeds `{allowed:?}`"
            ),
            Self::EmptySeparationId => formatter.write_str("separation rule id is empty"),
            Self::InvalidSeparationRule { roles, maximum } => write!(
                formatter,
                "separation rule has {roles} roles and invalid maximum {maximum}"
            ),
            Self::SeparationViolation {
                principal,
                rule,
                active_roles,
            } => write!(
                formatter,
                "principal `{principal}` violates separation rule `{rule}` with {active_roles:?}"
            ),
        }
    }
}

impl std::error::Error for AccessError {}

/// Immutable access policy graph.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccessPolicy {
    roles: BTreeSet<RoleId>,
    inheritance: BTreeMap<RoleId, BTreeSet<RoleId>>,
    grants: BTreeMap<GrantId, AccessGrant>,
    assignments: Vec<RoleAssignment>,
    delegations: BTreeMap<DelegationId, Delegation>,
    separation: BTreeMap<String, SeparationRule>,
}

impl AccessPolicy {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_role(&mut self, role: RoleId) -> Result<(), AccessError> {
        if !self.roles.insert(role.clone()) {
            return Err(AccessError::DuplicateRole(role));
        }
        Ok(())
    }

    /// Makes `role` inherit every permission of `parent`.
    pub fn inherit(&mut self, role: RoleId, parent: RoleId) -> Result<(), AccessError> {
        self.require_role(&role)?;
        self.require_role(&parent)?;
        self.inheritance
            .entry(role.clone())
            .or_default()
            .insert(parent.clone());
        if let Err(error) = self.validate_role_cycles() {
            self.inheritance
                .get_mut(&role)
                .expect("inserted inheritance")
                .remove(&parent);
            return Err(error);
        }
        Ok(())
    }

    pub fn add_grant(&mut self, grant: AccessGrant) -> Result<(), AccessError> {
        self.require_role(grant.role())?;
        if self.grants.contains_key(grant.id()) {
            return Err(AccessError::DuplicateGrant(grant.id().clone()));
        }
        self.grants.insert(grant.id().clone(), grant);
        Ok(())
    }

    pub fn assign(&mut self, assignment: RoleAssignment) -> Result<(), AccessError> {
        self.require_role(assignment.role())?;
        let mut candidate = self.assignments.clone();
        candidate.push(assignment.clone());
        self.validate_separation_for(
            assignment.principal(),
            assignment.valid_from,
            &candidate,
            &self.delegations,
        )?;
        self.assignments.push(assignment);
        Ok(())
    }

    pub fn add_separation_rule(&mut self, rule: SeparationRule) -> Result<(), AccessError> {
        for role in rule.roles() {
            self.require_role(role)?;
        }
        self.separation.insert(rule.id().to_owned(), rule);
        Ok(())
    }

    /// Adds a delegation after proving the delegator holds the role and the scope is bounded.
    pub fn delegate(&mut self, delegation: Delegation) -> Result<(), AccessError> {
        self.require_role(delegation.role())?;
        if self.delegations.contains_key(delegation.id()) {
            return Err(AccessError::DuplicateDelegation(delegation.id().clone()));
        }
        let delegator_roles = self.effective_roles(
            delegation.delegator(),
            delegation.valid_from,
            &AccessResourceId::new(delegation.scope().as_str())?,
        );
        if !delegator_roles.contains_key(delegation.role()) {
            return Err(AccessError::DelegatorLacksRole {
                delegator: delegation.delegator().clone(),
                role: delegation.role().clone(),
            });
        }
        self.delegations
            .insert(delegation.id().clone(), delegation.clone());
        if let Err(error) = self.validate_separation_for(
            delegation.delegate(),
            delegation.valid_from,
            &self.assignments,
            &self.delegations,
        ) {
            self.delegations.remove(delegation.id());
            return Err(error);
        }
        Ok(())
    }

    /// Evaluates permission using active assignments, inheritance, delegations, scope, and constraints.
    #[must_use]
    pub fn evaluate(
        &self,
        principal: &AccessPrincipalId,
        permission: &PermissionId,
        resource: &AccessResourceId,
        logical_time: u64,
        context: &Observation,
    ) -> AccessDecision {
        let roles = self.effective_roles(principal, logical_time, resource);
        let mut candidates = self
            .grants
            .values()
            .filter_map(|grant| {
                roles
                    .get(grant.role())
                    .map(|path| (grant, path.clone()))
            })
            .filter(|(grant, _)| grant.permission() == permission)
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(grant, _)| {
            (
                Reverse(grant.scope().specificity()),
                Reverse(grant.priority_value()),
                Reverse(grant.effect()),
                grant.id().clone(),
            )
        });

        let mut evaluations = Vec::with_capacity(candidates.len());
        let mut matched = Vec::new();
        for (grant, role_path) in candidates {
            let scope_matched = grant.scope().matches(resource);
            let mut failures = Vec::new();
            if !scope_matched {
                failures.push(format!(
                    "scope `{}` does not cover `{}`",
                    grant.scope().as_str(),
                    resource.as_str()
                ));
            }
            if scope_matched {
                for constraint in grant.constraints() {
                    if let Err(failure) = constraint.evaluate(context) {
                        failures.push(failure);
                    }
                }
            }
            let constraints_matched = failures.is_empty();
            let mut encoder = CanonicalEncoder::new();
            encoder
                .text("type", "grant-evaluation-v1")
                .field("grant", &grant.digest().0)
                .text("principal", principal.as_str())
                .text("resource", resource.as_str())
                .boolean("scope-matched", scope_matched)
                .boolean("constraints-matched", constraints_matched)
                .u64("failure-count", failures.len() as u64)
                .u64("role-path-count", role_path.len() as u64);
            for role in &role_path {
                encoder.text("role", role.as_str());
            }
            for failure in &failures {
                encoder.text("failure", failure);
            }
            evaluations.push(GrantEvaluation {
                grant: grant.id().clone(),
                role: grant.role().clone(),
                role_path: role_path.clone(),
                scope_matched,
                constraints_matched,
                failures,
                digest: encoder.digest(),
            });
            if scope_matched && constraints_matched {
                matched.push((grant, role_path));
            }
        }

        let selected = matched.first().cloned();
        let (kind, grant, role_path, code) = match selected {
            Some((grant, path)) if grant.effect() == AccessEffect::Allow => {
                ("allowed", Some(grant.id().clone()), path, None)
            }
            Some((grant, _)) => (
                "denied",
                Some(grant.id().clone()),
                Vec::new(),
                Some("EXPLICIT_DENY"),
            ),
            None => ("denied", None, Vec::new(), Some("NO_MATCHING_GRANT")),
        };
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "access-decision-v1")
            .field("policy", &self.digest().0)
            .text("kind", kind)
            .text("principal", principal.as_str())
            .text("permission", permission.as_str())
            .text("resource", resource.as_str())
            .u64("logical-time", logical_time)
            .field("context", &context.digest().0)
            .u64("evaluation-count", evaluations.len() as u64);
        for evaluation in &evaluations {
            encoder.field("evaluation", &evaluation.digest.0);
        }
        match &grant {
            Some(value) => {
                encoder
                    .boolean("has-grant", true)
                    .text("grant", value.as_str());
            }
            None => {
                encoder.boolean("has-grant", false);
            }
        }
        let digest = encoder.digest();
        if kind == "allowed" {
            AccessDecision::Allowed {
                grant: grant.expect("allowed decision has grant"),
                role_path,
                evaluations,
                digest,
            }
        } else {
            AccessDecision::Denied {
                code: code.expect("denied decision has code"),
                grant,
                evaluations,
                digest,
            }
        }
    }

    /// Returns each effective role and the shortest inheritance/delegation witness path.
    #[must_use]
    pub fn effective_roles(
        &self,
        principal: &AccessPrincipalId,
        logical_time: u64,
        resource: &AccessResourceId,
    ) -> BTreeMap<RoleId, Vec<RoleId>> {
        let mut roots = BTreeSet::new();
        for assignment in &self.assignments {
            if assignment.principal() == principal && assignment.active_at(logical_time) {
                roots.insert(assignment.role().clone());
            }
        }
        for delegation in self.delegations.values() {
            if delegation.delegate() == principal
                && delegation.active_at(logical_time)
                && delegation.scope().matches(resource)
            {
                roots.insert(delegation.role().clone());
            }
        }

        let mut result = BTreeMap::new();
        let mut queue = VecDeque::new();
        for root in roots {
            result.insert(root.clone(), vec![root.clone()]);
            queue.push_back(root);
        }
        while let Some(role) = queue.pop_front() {
            let path = result[&role].clone();
            for parent in self.inheritance.get(&role).into_iter().flatten() {
                let mut candidate = path.clone();
                candidate.push(parent.clone());
                let replace = result
                    .get(parent)
                    .is_none_or(|existing| candidate.len() < existing.len());
                if replace {
                    result.insert(parent.clone(), candidate);
                    queue.push_back(parent.clone());
                }
            }
        }
        result
    }

    /// Verifies role topology and all active separation constraints at a logical time.
    pub fn validate(&self, logical_time: u64) -> Result<(), AccessError> {
        self.validate_role_cycles()?;
        let principals = self
            .assignments
            .iter()
            .map(|assignment| assignment.principal().clone())
            .chain(
                self.delegations
                    .values()
                    .map(|delegation| delegation.delegate().clone()),
            )
            .collect::<BTreeSet<_>>();
        for principal in principals {
            self.validate_separation_for(
                &principal,
                logical_time,
                &self.assignments,
                &self.delegations,
            )?;
        }
        Ok(())
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "access-policy-v1")
            .u64("role-count", self.roles.len() as u64);
        for role in &self.roles {
            encoder.text("role", role.as_str());
            let parents = self.inheritance.get(role).cloned().unwrap_or_default();
            encoder.u64("parent-count", parents.len() as u64);
            for parent in parents {
                encoder.text("parent", parent.as_str());
            }
        }
        encoder.u64("grant-count", self.grants.len() as u64);
        for grant in self.grants.values() {
            encoder.field("grant", &grant.digest().0);
        }
        encoder.u64("assignment-count", self.assignments.len() as u64);
        for assignment in &self.assignments {
            encoder
                .text("assignment-principal", assignment.principal().as_str())
                .text("assignment-role", assignment.role().as_str())
                .u64("valid-from", assignment.valid_from);
            match assignment.valid_until {
                Some(value) => {
                    encoder.boolean("has-valid-until", true).u64("valid-until", value);
                }
                None => {
                    encoder.boolean("has-valid-until", false);
                }
            }
        }
        encoder.u64("delegation-count", self.delegations.len() as u64);
        for delegation in self.delegations.values() {
            encoder
                .text("delegation", delegation.id().as_str())
                .text("delegator", delegation.delegator().as_str())
                .text("delegate", delegation.delegate().as_str())
                .text("role", delegation.role().as_str())
                .text("scope", delegation.scope().as_str())
                .u64("valid-from", delegation.valid_from)
                .u64("valid-until", delegation.valid_until);
        }
        encoder.u64("separation-count", self.separation.len() as u64);
        for rule in self.separation.values() {
            encoder
                .text("separation", rule.id())
                .u64("maximum", rule.maximum() as u64)
                .u64("role-count", rule.roles().len() as u64);
            for role in rule.roles() {
                encoder.text("role", role.as_str());
            }
        }
        encoder.digest()
    }

    fn require_role(&self, role: &RoleId) -> Result<(), AccessError> {
        if self.roles.contains(role) {
            Ok(())
        } else {
            Err(AccessError::UnknownRole(role.clone()))
        }
    }

    fn validate_role_cycles(&self) -> Result<(), AccessError> {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum Mark {
            Visiting,
            Complete,
        }

        fn visit(
            policy: &AccessPolicy,
            role: &RoleId,
            marks: &mut BTreeMap<RoleId, Mark>,
            stack: &mut Vec<RoleId>,
        ) -> Result<(), AccessError> {
            match marks.get(role) {
                Some(Mark::Complete) => return Ok(()),
                Some(Mark::Visiting) => {
                    let start = stack.iter().position(|entry| entry == role).unwrap_or(0);
                    let mut path = stack[start..].to_vec();
                    path.push(role.clone());
                    return Err(AccessError::RoleCycle(path));
                }
                None => {}
            }
            marks.insert(role.clone(), Mark::Visiting);
            stack.push(role.clone());
            for parent in policy.inheritance.get(role).into_iter().flatten() {
                visit(policy, parent, marks, stack)?;
            }
            stack.pop();
            marks.insert(role.clone(), Mark::Complete);
            Ok(())
        }

        let mut marks = BTreeMap::new();
        let mut stack = Vec::new();
        for role in &self.roles {
            visit(self, role, &mut marks, &mut stack)?;
        }
        Ok(())
    }

    fn validate_separation_for(
        &self,
        principal: &AccessPrincipalId,
        logical_time: u64,
        assignments: &[RoleAssignment],
        delegations: &BTreeMap<DelegationId, Delegation>,
    ) -> Result<(), AccessError> {
        let mut active = assignments
            .iter()
            .filter(|assignment| {
                assignment.principal() == principal && assignment.active_at(logical_time)
            })
            .map(|assignment| assignment.role().clone())
            .collect::<BTreeSet<_>>();
        active.extend(
            delegations
                .values()
                .filter(|delegation| {
                    delegation.delegate() == principal && delegation.active_at(logical_time)
                })
                .map(|delegation| delegation.role().clone()),
        );
        for rule in self.separation.values() {
            let violating = active
                .intersection(rule.roles())
                .cloned()
                .collect::<Vec<_>>();
            if violating.len() > rule.maximum() {
                return Err(AccessError::SeparationViolation {
                    principal: principal.clone(),
                    rule: rule.id().to_owned(),
                    active_roles: violating,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AccessConstraint, AccessDecision, AccessEffect, AccessGrant, AccessPolicy,
        AccessPrincipalId, AccessResourceId, Delegation, DelegationId, GrantId, PermissionId,
        ResourceScope, RoleAssignment, RoleId, SeparationRule,
    };
    use crate::model::{Observation, SubjectId};
    use std::collections::BTreeSet;

    fn role(value: &str) -> RoleId {
        RoleId::new(value).unwrap()
    }

    fn principal(value: &str) -> AccessPrincipalId {
        AccessPrincipalId::new(value).unwrap()
    }

    fn resource(value: &str) -> AccessResourceId {
        AccessResourceId::new(value).unwrap()
    }

    fn policy() -> AccessPolicy {
        let mut policy = AccessPolicy::new();
        for value in ["viewer", "editor", "release"] {
            policy.add_role(role(value)).unwrap();
        }
        policy.inherit(role("editor"), role("viewer")).unwrap();
        policy
            .add_grant(AccessGrant::new(
                GrantId::new("view-all").unwrap(),
                role("viewer"),
                PermissionId::new("read").unwrap(),
                ResourceScope::new("/projects").unwrap(),
                AccessEffect::Allow,
            ))
            .unwrap();
        policy
            .add_grant(
                AccessGrant::new(
                    GrantId::new("edit-prod").unwrap(),
                    role("editor"),
                    PermissionId::new("write").unwrap(),
                    ResourceScope::new("/projects/prod").unwrap(),
                    AccessEffect::Allow,
                )
                .constrained_by(AccessConstraint::FactEquals {
                    key: "change-window".to_owned(),
                    expected: true.into(),
                }),
            )
            .unwrap();
        policy
    }

    #[test]
    fn inheritance_returns_shortest_permission_witness() {
        let mut policy = policy();
        policy
            .assign(RoleAssignment::new(principal("alice"), role("editor"), 0, None).unwrap())
            .unwrap();
        let context = Observation::new(SubjectId::new("request").unwrap(), 1);
        let AccessDecision::Allowed { role_path, .. } = policy.evaluate(
            &principal("alice"),
            &PermissionId::new("read").unwrap(),
            &resource("/projects/prod/app"),
            1,
            &context,
        ) else {
            panic!("editor must inherit viewer");
        };
        assert_eq!(
            role_path.iter().map(RoleId::as_str).collect::<Vec<_>>(),
            ["editor", "viewer"]
        );
    }

    #[test]
    fn contextual_grant_refuses_then_allows() {
        let mut policy = policy();
        policy
            .assign(RoleAssignment::new(principal("alice"), role("editor"), 0, None).unwrap())
            .unwrap();
        let mut context = Observation::new(SubjectId::new("request").unwrap(), 1);
        assert!(!policy
            .evaluate(
                &principal("alice"),
                &PermissionId::new("write").unwrap(),
                &resource("/projects/prod/app"),
                1,
                &context,
            )
            .allowed());
        context.insert("change-window", true).unwrap();
        assert!(policy
            .evaluate(
                &principal("alice"),
                &PermissionId::new("write").unwrap(),
                &resource("/projects/prod/app"),
                1,
                &context,
            )
            .allowed());
    }

    #[test]
    fn deny_overrides_allow_at_more_specific_scope() {
        let mut policy = policy();
        policy
            .add_grant(
                AccessGrant::new(
                    GrantId::new("deny-secret").unwrap(),
                    role("viewer"),
                    PermissionId::new("read").unwrap(),
                    ResourceScope::new("/projects/prod/secret").unwrap(),
                    AccessEffect::Deny,
                )
                .priority(10),
            )
            .unwrap();
        policy
            .assign(RoleAssignment::new(principal("alice"), role("viewer"), 0, None).unwrap())
            .unwrap();
        let context = Observation::new(SubjectId::new("request").unwrap(), 1);
        let decision = policy.evaluate(
            &principal("alice"),
            &PermissionId::new("read").unwrap(),
            &resource("/projects/prod/secret/key"),
            1,
            &context,
        );
        assert!(matches!(
            decision,
            AccessDecision::Denied {
                code: "EXPLICIT_DENY",
                ..
            }
        ));
    }

    #[test]
    fn bounded_delegation_activates_role() {
        let mut policy = policy();
        policy
            .assign(RoleAssignment::new(principal("alice"), role("viewer"), 0, None).unwrap())
            .unwrap();
        policy
            .delegate(
                Delegation::new(
                    DelegationId::new("delegate-1").unwrap(),
                    principal("alice"),
                    principal("bob"),
                    role("viewer"),
                    ResourceScope::new("/projects/prod").unwrap(),
                    1,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
        let context = Observation::new(SubjectId::new("request").unwrap(), 1);
        assert!(policy
            .evaluate(
                &principal("bob"),
                &PermissionId::new("read").unwrap(),
                &resource("/projects/prod/app"),
                2,
                &context,
            )
            .allowed());
        assert!(!policy
            .evaluate(
                &principal("bob"),
                &PermissionId::new("read").unwrap(),
                &resource("/projects/dev/app"),
                2,
                &context,
            )
            .allowed());
    }

    #[test]
    fn separation_of_duties_refuses_conflicting_assignment() {
        let mut policy = policy();
        policy
            .add_separation_rule(
                SeparationRule::new(
                    "maker-checker",
                    BTreeSet::from([role("editor"), role("release")]),
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        policy
            .assign(RoleAssignment::new(principal("alice"), role("editor"), 0, None).unwrap())
            .unwrap();
        assert!(policy
            .assign(RoleAssignment::new(principal("alice"), role("release"), 0, None).unwrap())
            .is_err());
    }
}
