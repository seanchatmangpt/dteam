//! Deterministic document schemas, validation, compatibility, and migration.

use crate::hash::{CanonicalEncoder, Digest};
use crate::model::{FactValue, Observation};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

/// Supported fact type.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValueType {
    Bool,
    I64,
    U64,
    Text,
    Bytes,
    TextSet,
}

impl ValueType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::Text => "text",
            Self::Bytes => "bytes",
            Self::TextSet => "text-set",
        }
    }

    #[must_use]
    pub const fn matches(self, value: &FactValue) -> bool {
        matches!(
            (self, value),
            (Self::Bool, FactValue::Bool(_))
                | (Self::I64, FactValue::I64(_))
                | (Self::U64, FactValue::U64(_))
                | (Self::Text, FactValue::Text(_))
                | (Self::Bytes, FactValue::Bytes(_))
                | (Self::TextSet, FactValue::TextSet(_))
        )
    }
}

/// Closed deterministic field constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    I64Range {
        minimum: Option<i64>,
        maximum: Option<i64>,
    },
    U64Range {
        minimum: Option<u64>,
        maximum: Option<u64>,
    },
    TextLength {
        minimum: usize,
        maximum: Option<usize>,
    },
    TextOneOf(BTreeSet<String>),
    TextPrefix(String),
    BytesLength {
        minimum: usize,
        maximum: Option<usize>,
    },
    TextSetCardinality {
        minimum: usize,
        maximum: Option<usize>,
    },
    TextSetAllowed(BTreeSet<String>),
}

impl Constraint {
    fn validate(&self, key: &str, value: &FactValue) -> Option<ValidationIssue> {
        let issue = |code, detail| {
            Some(ValidationIssue {
                key: key.to_owned(),
                code,
                detail,
            })
        };
        match self {
            Self::I64Range { minimum, maximum } => match value {
                FactValue::I64(actual) => {
                    if minimum.is_some_and(|bound| *actual < bound) {
                        issue(
                            "BELOW_MINIMUM",
                            format!("value {actual} is below {minimum:?}"),
                        )
                    } else if maximum.is_some_and(|bound| *actual > bound) {
                        issue(
                            "ABOVE_MAXIMUM",
                            format!("value {actual} is above {maximum:?}"),
                        )
                    } else {
                        None
                    }
                }
                _ => issue("CONSTRAINT_TYPE", "i64 range requires i64".to_owned()),
            },
            Self::U64Range { minimum, maximum } => match value {
                FactValue::U64(actual) => {
                    if minimum.is_some_and(|bound| *actual < bound) {
                        issue(
                            "BELOW_MINIMUM",
                            format!("value {actual} is below {minimum:?}"),
                        )
                    } else if maximum.is_some_and(|bound| *actual > bound) {
                        issue(
                            "ABOVE_MAXIMUM",
                            format!("value {actual} is above {maximum:?}"),
                        )
                    } else {
                        None
                    }
                }
                _ => issue("CONSTRAINT_TYPE", "u64 range requires u64".to_owned()),
            },
            Self::TextLength { minimum, maximum } => match value {
                FactValue::Text(actual) => {
                    let length = actual.chars().count();
                    if length < *minimum {
                        issue(
                            "TEXT_TOO_SHORT",
                            format!("text length {length} is below {minimum}"),
                        )
                    } else if maximum.is_some_and(|bound| length > bound) {
                        issue(
                            "TEXT_TOO_LONG",
                            format!("text length {length} exceeds {maximum:?}"),
                        )
                    } else {
                        None
                    }
                }
                _ => issue("CONSTRAINT_TYPE", "text length requires text".to_owned()),
            },
            Self::TextOneOf(allowed) => match value {
                FactValue::Text(actual) if allowed.contains(actual) => None,
                FactValue::Text(actual) => issue(
                    "VALUE_NOT_ALLOWED",
                    format!("text `{actual}` is not in {allowed:?}"),
                ),
                _ => issue("CONSTRAINT_TYPE", "text allowlist requires text".to_owned()),
            },
            Self::TextPrefix(prefix) => match value {
                FactValue::Text(actual) if actual.starts_with(prefix) => None,
                FactValue::Text(actual) => issue(
                    "PREFIX_MISMATCH",
                    format!("text `{actual}` does not start with `{prefix}`"),
                ),
                _ => issue("CONSTRAINT_TYPE", "text prefix requires text".to_owned()),
            },
            Self::BytesLength { minimum, maximum } => match value {
                FactValue::Bytes(actual) => {
                    if actual.len() < *minimum {
                        issue(
                            "BYTES_TOO_SHORT",
                            format!("byte length {} is below {minimum}", actual.len()),
                        )
                    } else if maximum.is_some_and(|bound| actual.len() > bound) {
                        issue(
                            "BYTES_TOO_LONG",
                            format!("byte length {} exceeds {maximum:?}", actual.len()),
                        )
                    } else {
                        None
                    }
                }
                _ => issue("CONSTRAINT_TYPE", "byte length requires bytes".to_owned()),
            },
            Self::TextSetCardinality { minimum, maximum } => match value {
                FactValue::TextSet(actual) => {
                    if actual.len() < *minimum {
                        issue(
                            "SET_TOO_SMALL",
                            format!("set size {} is below {minimum}", actual.len()),
                        )
                    } else if maximum.is_some_and(|bound| actual.len() > bound) {
                        issue(
                            "SET_TOO_LARGE",
                            format!("set size {} exceeds {maximum:?}", actual.len()),
                        )
                    } else {
                        None
                    }
                }
                _ => issue(
                    "CONSTRAINT_TYPE",
                    "set cardinality requires text set".to_owned(),
                ),
            },
            Self::TextSetAllowed(allowed) => match value {
                FactValue::TextSet(actual) => {
                    let forbidden = actual.difference(allowed).cloned().collect::<Vec<_>>();
                    if forbidden.is_empty() {
                        None
                    } else {
                        issue(
                            "SET_MEMBER_NOT_ALLOWED",
                            format!("members {forbidden:?} are not allowed"),
                        )
                    }
                }
                _ => issue(
                    "CONSTRAINT_TYPE",
                    "set allowlist requires text set".to_owned(),
                ),
            },
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::I64Range { minimum, maximum } => {
                encoder.text("constraint", "i64-range");
                encode_i64_bounds(encoder, *minimum, *maximum);
            }
            Self::U64Range { minimum, maximum } => {
                encoder.text("constraint", "u64-range");
                encode_u64_bounds(encoder, *minimum, *maximum);
            }
            Self::TextLength { minimum, maximum } => {
                encoder
                    .text("constraint", "text-length")
                    .u64("minimum", *minimum as u64);
                encode_usize_maximum(encoder, *maximum);
            }
            Self::TextOneOf(allowed) => {
                encoder
                    .text("constraint", "text-one-of")
                    .u64("allowed-count", allowed.len() as u64);
                for value in allowed {
                    encoder.text("allowed", value);
                }
            }
            Self::TextPrefix(prefix) => {
                encoder
                    .text("constraint", "text-prefix")
                    .text("prefix", prefix);
            }
            Self::BytesLength { minimum, maximum } => {
                encoder
                    .text("constraint", "bytes-length")
                    .u64("minimum", *minimum as u64);
                encode_usize_maximum(encoder, *maximum);
            }
            Self::TextSetCardinality { minimum, maximum } => {
                encoder
                    .text("constraint", "text-set-cardinality")
                    .u64("minimum", *minimum as u64);
                encode_usize_maximum(encoder, *maximum);
            }
            Self::TextSetAllowed(allowed) => {
                encoder
                    .text("constraint", "text-set-allowed")
                    .u64("allowed-count", allowed.len() as u64);
                for value in allowed {
                    encoder.text("allowed", value);
                }
            }
        }
    }
}

fn encode_i64_bounds(encoder: &mut CanonicalEncoder, minimum: Option<i64>, maximum: Option<i64>) {
    match minimum {
        Some(value) => {
            encoder.boolean("has-minimum", true).i64("minimum", value);
        }
        None => {
            encoder.boolean("has-minimum", false);
        }
    }
    match maximum {
        Some(value) => {
            encoder.boolean("has-maximum", true).i64("maximum", value);
        }
        None => {
            encoder.boolean("has-maximum", false);
        }
    }
}

fn encode_u64_bounds(encoder: &mut CanonicalEncoder, minimum: Option<u64>, maximum: Option<u64>) {
    match minimum {
        Some(value) => {
            encoder.boolean("has-minimum", true).u64("minimum", value);
        }
        None => {
            encoder.boolean("has-minimum", false);
        }
    }
    match maximum {
        Some(value) => {
            encoder.boolean("has-maximum", true).u64("maximum", value);
        }
        None => {
            encoder.boolean("has-maximum", false);
        }
    }
}

fn encode_usize_maximum(encoder: &mut CanonicalEncoder, maximum: Option<usize>) {
    match maximum {
        Some(value) => {
            encoder
                .boolean("has-maximum", true)
                .u64("maximum", value as u64);
        }
        None => {
            encoder.boolean("has-maximum", false);
        }
    }
}

/// One field declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldSchema {
    name: String,
    value_type: ValueType,
    required: bool,
    default: Option<FactValue>,
    constraints: Vec<Constraint>,
}

impl FieldSchema {
    /// Creates a field declaration.
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Result<Self, SchemaError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(SchemaError::EmptyFieldName);
        }
        Ok(Self {
            name,
            value_type,
            required: false,
            default: None,
            constraints: Vec::new(),
        })
    }

    #[must_use]
    pub const fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    pub fn default(mut self, value: FactValue) -> Result<Self, SchemaError> {
        if !self.value_type.matches(&value) {
            return Err(SchemaError::DefaultType {
                field: self.name.clone(),
                expected: self.value_type,
            });
        }
        self.default = Some(value);
        Ok(self)
    }

    #[must_use]
    pub fn constrained_by(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn value_type(&self) -> ValueType {
        self.value_type
    }

    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }

    #[must_use]
    pub fn default_value(&self) -> Option<&FactValue> {
        self.default.as_ref()
    }

    #[must_use]
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "field-schema-v1")
            .text("name", &self.name)
            .text("value-type", self.value_type.as_str())
            .boolean("required", self.required);
        match &self.default {
            Some(value) => {
                encoder.boolean("has-default", true);
                value.encode(&mut encoder, "default-type");
            }
            None => {
                encoder.boolean("has-default", false);
            }
        }
        encoder.u64("constraint-count", self.constraints.len() as u64);
        for constraint in &self.constraints {
            constraint.encode(&mut encoder);
        }
        encoder.digest()
    }
}

/// Policy for undeclared fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnknownFieldPolicy {
    Allow,
    Refuse,
    Drop,
}

impl UnknownFieldPolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Refuse => "refuse",
            Self::Drop => "drop",
        }
    }
}

/// Deterministic fact document.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Document {
    fields: BTreeMap<String, FactValue>,
}

impl Document {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Copies all facts from an observation.
    #[must_use]
    pub fn from_observation(observation: &Observation) -> Self {
        Self {
            fields: observation
                .facts()
                .map(|(key, value)| (key.to_owned(), value.clone()))
                .collect(),
        }
    }

    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) -> Option<FactValue> {
        self.fields.insert(key.into(), value.into())
    }

    pub fn remove(&mut self, key: &str) -> Option<FactValue> {
        self.fields.remove(key)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FactValue> {
        self.fields.get(key)
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = (&str, &FactValue)> {
        self.fields.iter().map(|(key, value)| (key.as_str(), value))
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "schema-document-v1")
            .u64("field-count", self.fields.len() as u64);
        for (key, value) in &self.fields {
            encoder.text("field", key);
            value.encode(&mut encoder, "value-type");
        }
        encoder.digest()
    }
}

/// Versioned deterministic schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSchema {
    id: String,
    version: u64,
    unknown_fields: UnknownFieldPolicy,
    fields: BTreeMap<String, FieldSchema>,
}

impl DocumentSchema {
    pub fn new(id: impl Into<String>, version: u64) -> Result<Self, SchemaError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(SchemaError::EmptySchemaId);
        }
        Ok(Self {
            id,
            version,
            unknown_fields: UnknownFieldPolicy::Refuse,
            fields: BTreeMap::new(),
        })
    }

    #[must_use]
    pub const fn unknown_fields(mut self, value: UnknownFieldPolicy) -> Self {
        self.unknown_fields = value;
        self
    }

    pub fn add_field(&mut self, field: FieldSchema) -> Result<(), SchemaError> {
        if self.fields.contains_key(field.name()) {
            return Err(SchemaError::DuplicateField(field.name().to_owned()));
        }
        if let Some(default) = field.default_value() {
            for constraint in field.constraints() {
                if let Some(issue) = constraint.validate(field.name(), default) {
                    return Err(SchemaError::InvalidDefault {
                        field: field.name().to_owned(),
                        issue,
                    });
                }
            }
        }
        self.fields.insert(field.name().to_owned(), field);
        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    pub const fn unknown_field_policy(&self) -> UnknownFieldPolicy {
        self.unknown_fields
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<&FieldSchema> {
        self.fields.get(name)
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = &FieldSchema> {
        self.fields.values()
    }

    /// Validates and normalizes a document by applying defaults and unknown-field policy.
    #[must_use]
    pub fn validate(&self, input: &Document) -> ValidationReport {
        let mut normalized = input.clone();
        let mut issues = Vec::new();
        let unknown = input
            .fields
            .keys()
            .filter(|key| !self.fields.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        match self.unknown_fields {
            UnknownFieldPolicy::Allow => {}
            UnknownFieldPolicy::Refuse => {
                for key in unknown {
                    issues.push(ValidationIssue {
                        key,
                        code: "UNKNOWN_FIELD",
                        detail: "field is not declared by the schema".to_owned(),
                    });
                }
            }
            UnknownFieldPolicy::Drop => {
                for key in unknown {
                    normalized.remove(&key);
                }
            }
        }

        for field in self.fields.values() {
            match normalized.get(field.name()) {
                Some(value) => {
                    if !field.value_type().matches(value) {
                        issues.push(ValidationIssue {
                            key: field.name().to_owned(),
                            code: "TYPE_MISMATCH",
                            detail: format!(
                                "expected {}, found {value:?}",
                                field.value_type().as_str()
                            ),
                        });
                        continue;
                    }
                    for constraint in field.constraints() {
                        if let Some(issue) = constraint.validate(field.name(), value) {
                            issues.push(issue);
                        }
                    }
                }
                None => {
                    if let Some(default) = field.default_value() {
                        normalized.insert(field.name().to_owned(), default.clone());
                    } else if field.is_required() {
                        issues.push(ValidationIssue {
                            key: field.name().to_owned(),
                            code: "REQUIRED_FIELD",
                            detail: "required field is absent".to_owned(),
                        });
                    }
                }
            }
        }

        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "validation-report-v1")
            .field("schema", &self.digest().0)
            .field("input", &input.digest().0)
            .field("normalized", &normalized.digest().0)
            .u64("issue-count", issues.len() as u64);
        for issue in &issues {
            encoder
                .text("key", issue.key())
                .text("code", issue.code())
                .text("detail", issue.detail());
        }
        ValidationReport {
            normalized,
            issues,
            digest: encoder.digest(),
        }
    }

    /// Computes source compatibility for readers expecting `self` and receiving `candidate`.
    #[must_use]
    pub fn compatibility_with(&self, candidate: &DocumentSchema) -> CompatibilityReport {
        let mut changes = Vec::new();
        if self.id != candidate.id {
            changes.push(CompatibilityChange::SchemaIdentityChanged {
                from: self.id.clone(),
                to: candidate.id.clone(),
            });
        }
        for (name, existing) in &self.fields {
            match candidate.fields.get(name) {
                None => changes.push(CompatibilityChange::FieldRemoved {
                    field: name.clone(),
                    breaking: existing.is_required(),
                }),
                Some(next) => {
                    if existing.value_type() != next.value_type() {
                        changes.push(CompatibilityChange::TypeChanged {
                            field: name.clone(),
                            from: existing.value_type(),
                            to: next.value_type(),
                        });
                    }
                    if !existing.is_required()
                        && next.is_required()
                        && next.default_value().is_none()
                    {
                        changes.push(CompatibilityChange::FieldBecameRequired {
                            field: name.clone(),
                        });
                    }
                    if existing.constraints() != next.constraints() {
                        changes.push(CompatibilityChange::ConstraintsChanged {
                            field: name.clone(),
                        });
                    }
                }
            }
        }
        for (name, added) in &candidate.fields {
            if !self.fields.contains_key(name) {
                changes.push(CompatibilityChange::FieldAdded {
                    field: name.clone(),
                    breaking: added.is_required() && added.default_value().is_none(),
                });
            }
        }
        if self.unknown_fields == UnknownFieldPolicy::Allow
            && candidate.unknown_fields == UnknownFieldPolicy::Refuse
        {
            changes.push(CompatibilityChange::UnknownFieldsClosed);
        }
        let compatible = changes.iter().all(|change| !change.breaking());
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "schema-compatibility-v1")
            .field("from", &self.digest().0)
            .field("to", &candidate.digest().0)
            .boolean("compatible", compatible)
            .u64("change-count", changes.len() as u64);
        for change in &changes {
            change.encode(&mut encoder);
        }
        CompatibilityReport {
            compatible,
            changes,
            digest: encoder.digest(),
        }
    }

    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "document-schema-v1")
            .text("id", &self.id)
            .u64("version", self.version)
            .text("unknown-fields", self.unknown_fields.as_str())
            .u64("field-count", self.fields.len() as u64);
        for field in self.fields.values() {
            encoder
                .text("field", field.name())
                .field("field-digest", &field.digest().0);
        }
        encoder.digest()
    }
}

/// Schema construction failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaError {
    EmptySchemaId,
    EmptyFieldName,
    DuplicateField(String),
    DefaultType {
        field: String,
        expected: ValueType,
    },
    InvalidDefault {
        field: String,
        issue: ValidationIssue,
    },
    DuplicateMigrationTarget(String),
    MissingMigrationSource(String),
    MigrationTargetExists(String),
}

impl Display for SchemaError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySchemaId => formatter.write_str("schema id must not be empty"),
            Self::EmptyFieldName => formatter.write_str("field name must not be empty"),
            Self::DuplicateField(field) => write!(formatter, "duplicate field `{field}`"),
            Self::DefaultType { field, expected } => write!(
                formatter,
                "default for `{field}` does not match {}",
                expected.as_str()
            ),
            Self::InvalidDefault { field, issue } => {
                write!(formatter, "default for `{field}` is invalid: {issue}")
            }
            Self::DuplicateMigrationTarget(field) => {
                write!(formatter, "migration writes `{field}` more than once")
            }
            Self::MissingMigrationSource(field) => {
                write!(formatter, "migration source `{field}` is absent")
            }
            Self::MigrationTargetExists(field) => {
                write!(formatter, "migration target `{field}` already exists")
            }
        }
    }
}

impl std::error::Error for SchemaError {}

/// One validation defect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    key: String,
    code: &'static str,
    detail: String,
}

impl ValidationIssue {
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl Display for ValidationIssue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} [{}]: {}", self.key, self.code, self.detail)
    }
}

/// Complete normalized validation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    normalized: Document,
    issues: Vec<ValidationIssue>,
    digest: Digest,
}

impl ValidationReport {
    #[must_use]
    pub fn normalized(&self) -> &Document {
        &self.normalized
    }

    #[must_use]
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    #[must_use]
    pub fn valid(&self) -> bool {
        self.issues.is_empty()
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Schema compatibility delta.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompatibilityChange {
    SchemaIdentityChanged {
        from: String,
        to: String,
    },
    FieldRemoved {
        field: String,
        breaking: bool,
    },
    FieldAdded {
        field: String,
        breaking: bool,
    },
    TypeChanged {
        field: String,
        from: ValueType,
        to: ValueType,
    },
    FieldBecameRequired {
        field: String,
    },
    ConstraintsChanged {
        field: String,
    },
    UnknownFieldsClosed,
}

impl CompatibilityChange {
    #[must_use]
    pub const fn breaking(&self) -> bool {
        match self {
            Self::SchemaIdentityChanged { .. }
            | Self::TypeChanged { .. }
            | Self::FieldBecameRequired { .. }
            | Self::ConstraintsChanged { .. }
            | Self::UnknownFieldsClosed => true,
            Self::FieldRemoved { breaking, .. } | Self::FieldAdded { breaking, .. } => *breaking,
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::SchemaIdentityChanged { from, to } => {
                encoder
                    .text("change", "schema-identity")
                    .text("from", from)
                    .text("to", to);
            }
            Self::FieldRemoved { field, breaking } => {
                encoder
                    .text("change", "field-removed")
                    .text("field", field)
                    .boolean("breaking", *breaking);
            }
            Self::FieldAdded { field, breaking } => {
                encoder
                    .text("change", "field-added")
                    .text("field", field)
                    .boolean("breaking", *breaking);
            }
            Self::TypeChanged { field, from, to } => {
                encoder
                    .text("change", "type-changed")
                    .text("field", field)
                    .text("from", from.as_str())
                    .text("to", to.as_str());
            }
            Self::FieldBecameRequired { field } => {
                encoder
                    .text("change", "field-became-required")
                    .text("field", field);
            }
            Self::ConstraintsChanged { field } => {
                encoder
                    .text("change", "constraints-changed")
                    .text("field", field);
            }
            Self::UnknownFieldsClosed => {
                encoder.text("change", "unknown-fields-closed");
            }
        }
    }
}

/// Complete compatibility result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityReport {
    compatible: bool,
    changes: Vec<CompatibilityChange>,
    digest: Digest,
}

impl CompatibilityReport {
    #[must_use]
    pub const fn compatible(&self) -> bool {
        self.compatible
    }

    #[must_use]
    pub fn changes(&self) -> &[CompatibilityChange] {
        &self.changes
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

/// Pure document migration operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationStep {
    Rename { from: String, to: String },
    Copy { from: String, to: String },
    Drop { field: String },
    SetDefault { field: String, value: FactValue },
    Require { field: String },
}

impl MigrationStep {
    fn target(&self) -> Option<&str> {
        match self {
            Self::Rename { to, .. } | Self::Copy { to, .. } => Some(to),
            Self::SetDefault { field, .. } => Some(field),
            Self::Drop { .. } | Self::Require { .. } => None,
        }
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        match self {
            Self::Rename { from, to } => {
                encoder
                    .text("step", "rename")
                    .text("from", from)
                    .text("to", to);
            }
            Self::Copy { from, to } => {
                encoder
                    .text("step", "copy")
                    .text("from", from)
                    .text("to", to);
            }
            Self::Drop { field } => {
                encoder.text("step", "drop").text("field", field);
            }
            Self::SetDefault { field, value } => {
                encoder.text("step", "set-default").text("field", field);
                value.encode(encoder, "value-type");
            }
            Self::Require { field } => {
                encoder.text("step", "require").text("field", field);
            }
        }
    }
}

/// Deterministic migration plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationPlan {
    id: String,
    from_version: u64,
    to_version: u64,
    steps: Vec<MigrationStep>,
    digest: Digest,
}

impl MigrationPlan {
    pub fn new(
        id: impl Into<String>,
        from_version: u64,
        to_version: u64,
        steps: Vec<MigrationStep>,
    ) -> Result<Self, SchemaError> {
        let id = id.into();
        let mut targets = BTreeSet::new();
        for step in &steps {
            if let Some(target) = step.target() {
                if !targets.insert(target.to_owned()) {
                    return Err(SchemaError::DuplicateMigrationTarget(target.to_owned()));
                }
            }
        }
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "migration-plan-v1")
            .text("id", &id)
            .u64("from-version", from_version)
            .u64("to-version", to_version)
            .u64("step-count", steps.len() as u64);
        for step in &steps {
            step.encode(&mut encoder);
        }
        Ok(Self {
            id,
            from_version,
            to_version,
            steps,
            digest: encoder.digest(),
        })
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn from_version(&self) -> u64 {
        self.from_version
    }

    #[must_use]
    pub const fn to_version(&self) -> u64 {
        self.to_version
    }

    #[must_use]
    pub fn steps(&self) -> &[MigrationStep] {
        &self.steps
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    /// Applies all steps atomically to a clone of the input.
    pub fn apply(&self, input: &Document) -> Result<MigrationResult, SchemaError> {
        let mut output = input.clone();
        for step in &self.steps {
            match step {
                MigrationStep::Rename { from, to } => {
                    let value = output
                        .remove(from)
                        .ok_or_else(|| SchemaError::MissingMigrationSource(from.clone()))?;
                    if output.get(to).is_some() {
                        return Err(SchemaError::MigrationTargetExists(to.clone()));
                    }
                    output.insert(to.clone(), value);
                }
                MigrationStep::Copy { from, to } => {
                    let value = output
                        .get(from)
                        .cloned()
                        .ok_or_else(|| SchemaError::MissingMigrationSource(from.clone()))?;
                    if output.get(to).is_some() {
                        return Err(SchemaError::MigrationTargetExists(to.clone()));
                    }
                    output.insert(to.clone(), value);
                }
                MigrationStep::Drop { field } => {
                    output.remove(field);
                }
                MigrationStep::SetDefault { field, value } => {
                    if output.get(field).is_none() {
                        output.insert(field.clone(), value.clone());
                    }
                }
                MigrationStep::Require { field } => {
                    if output.get(field).is_none() {
                        return Err(SchemaError::MissingMigrationSource(field.clone()));
                    }
                }
            }
        }
        let mut encoder = CanonicalEncoder::new();
        encoder
            .text("type", "migration-result-v1")
            .field("plan", &self.digest.0)
            .field("input", &input.digest().0)
            .field("output", &output.digest().0);
        Ok(MigrationResult {
            output,
            digest: encoder.digest(),
        })
    }
}

/// Completed migration evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationResult {
    output: Document,
    digest: Digest,
}

impl MigrationResult {
    #[must_use]
    pub fn output(&self) -> &Document {
        &self.output
    }

    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompatibilityChange, Constraint, Document, DocumentSchema, FieldSchema, MigrationPlan,
        MigrationStep, UnknownFieldPolicy, ValueType,
    };
    use crate::model::FactValue;

    fn schema(version: u64) -> DocumentSchema {
        let mut schema = DocumentSchema::new("case", version)
            .unwrap()
            .unknown_fields(UnknownFieldPolicy::Drop);
        schema
            .add_field(
                FieldSchema::new("status", ValueType::Text)
                    .unwrap()
                    .required(true)
                    .constrained_by(Constraint::TextOneOf(
                        ["open".to_owned(), "closed".to_owned()]
                            .into_iter()
                            .collect(),
                    )),
            )
            .unwrap();
        schema
            .add_field(
                FieldSchema::new("priority", ValueType::U64)
                    .unwrap()
                    .default(1_u64.into())
                    .unwrap()
                    .constrained_by(Constraint::U64Range {
                        minimum: Some(1),
                        maximum: Some(5),
                    }),
            )
            .unwrap();
        schema
    }

    #[test]
    fn validation_applies_defaults_and_drops_unknown_fields() {
        let schema = schema(1);
        let mut document = Document::new();
        document.insert("status", "open");
        document.insert("unknown", true);
        let report = schema.validate(&document);
        assert!(report.valid());
        assert_eq!(
            report.normalized().get("priority"),
            Some(&FactValue::U64(1))
        );
        assert!(report.normalized().get("unknown").is_none());
    }

    #[test]
    fn validation_collects_type_and_constraint_issues() {
        let schema = schema(1);
        let mut document = Document::new();
        document.insert("status", "invalid");
        document.insert("priority", "high");
        let report = schema.validate(&document);
        assert_eq!(report.issues().len(), 2);
        assert!(report
            .issues()
            .iter()
            .any(|issue| issue.code() == "VALUE_NOT_ALLOWED"));
        assert!(report
            .issues()
            .iter()
            .any(|issue| issue.code() == "TYPE_MISMATCH"));
    }

    #[test]
    fn compatibility_reports_breaking_required_addition() {
        let original = schema(1);
        let mut next = schema(2);
        next.add_field(
            FieldSchema::new("owner", ValueType::Text)
                .unwrap()
                .required(true),
        )
        .unwrap();
        let report = original.compatibility_with(&next);
        assert!(!report.compatible());
        assert!(report.changes().iter().any(|change| matches!(
            change,
            CompatibilityChange::FieldAdded {
                field,
                breaking: true
            } if field == "owner"
        )));
    }

    #[test]
    fn migration_renames_and_defaults_atomically() {
        let mut input = Document::new();
        input.insert("state", "open");
        let plan = MigrationPlan::new(
            "v1-v2",
            1,
            2,
            vec![
                MigrationStep::Rename {
                    from: "state".to_owned(),
                    to: "status".to_owned(),
                },
                MigrationStep::SetDefault {
                    field: "priority".to_owned(),
                    value: 1_u64.into(),
                },
                MigrationStep::Require {
                    field: "status".to_owned(),
                },
            ],
        )
        .unwrap();
        let result = plan.apply(&input).unwrap();
        assert_eq!(
            result.output().get("status"),
            Some(&FactValue::Text("open".to_owned()))
        );
        assert_eq!(result.output().get("priority"), Some(&FactValue::U64(1)));
        assert!(input.get("state").is_some());
    }
}
