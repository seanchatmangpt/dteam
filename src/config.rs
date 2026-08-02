use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicConfig {
    pub meta: MetaConfig,
    pub kernel: KernelConfig,
    pub autonomic: AutonomicSystemConfig,
    pub rl: RlConfig,
    pub discovery: DiscoveryConfig,
    pub paths: PathConfig,
    pub wasm: WasmConfig,
    #[serde(default)]
    pub automl: AutomlConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    pub version: String,
    pub environment: String,
    pub identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    pub tier: String,
    pub alignment: usize,
    pub determinism: String,
    pub allocation_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicSystemConfig {
    pub mode: String,
    pub sampling_rate: u64,
    pub integrity_hash: String,
    pub guards: GuardConfig,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardConfig {
    pub risk_threshold: String,
    pub min_health_threshold: f32,
    pub max_cycle_latency_ms: u64,
    pub repair_authority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub profile: String,
    pub mdl_penalty: f32,
    pub human_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlConfig {
    pub algorithm: String,
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub exploration_rate: f32,
    pub exploration_decay: f32,
    pub reward_weights: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub max_training_epochs: usize,
    pub fitness_stopping_threshold: f64,
    pub strategy: String,
    pub drift_window: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub training_logs_dir: String,
    pub test_logs_dir: String,
    pub ground_truth_dir: String,
    pub artifacts_dir: String,
    pub manifest_bus_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub batch_size: usize,
    pub max_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomlConfig {
    pub enabled: bool,
    pub strategy: String,
    pub budget: usize,
    pub seed: u64,
    /// TPOT2 successive halving: enable 2-rung evaluation for HDIT signal selection.
    /// When true, rung-0 scores all signals on a subsample; only top fraction advance.
    #[serde(default)]
    pub successive_halving: bool,
    /// Rung-0 subsample fraction (e.g. 0.2 = 20% of traces). Only used if successive_halving=true.
    #[serde(default = "default_sh_subsample")]
    pub sh_subsample: f64,
    /// Promotion ratio: keep top 1/ratio candidates (e.g. 3.0 = top third).
    #[serde(default = "default_sh_ratio")]
    pub sh_promotion_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "path", rename_all = "snake_case")]
pub enum ConfigSource {
    Default,
    File(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedAutonomicConfig {
    pub config: AutonomicConfig,
    pub source: ConfigSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValidationError {
    violations: Vec<String>,
}

impl ConfigValidationError {
    pub fn violations(&self) -> &[String] {
        &self.violations
    }
}

impl fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid dteam configuration: {}", self.violations.join("; "))
    }
}

impl std::error::Error for ConfigValidationError {}

fn default_sh_subsample() -> f64 {
    0.2
}
fn default_sh_ratio() -> f64 {
    3.0
}

impl Default for AutomlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: "random".to_string(),
            budget: 20,
            seed: 42,
            successive_halving: false,
            sh_subsample: 0.2,
            sh_promotion_ratio: 3.0,
        }
    }
}

impl Default for AutonomicConfig {
    fn default() -> Self {
        let mut reward_weights = HashMap::new();
        reward_weights.insert("fitness".to_string(), 0.6);
        reward_weights.insert("soundness".to_string(), 0.2);
        reward_weights.insert("simplicity".to_string(), 0.1);
        reward_weights.insert("latency".to_string(), 0.1);

        Self {
            meta: MetaConfig {
                version: "2026.04.18".to_string(),
                environment: "autonomous".to_string(),
                identity: "dteam-alpha-01".to_string(),
            },
            kernel: KernelConfig {
                tier: "K256".to_string(),
                alignment: 8,
                determinism: "strict".to_string(),
                allocation_policy: "zero_heap".to_string(),
            },
            autonomic: AutonomicSystemConfig {
                mode: "guarded".to_string(),
                sampling_rate: 100,
                integrity_hash: "fnv1a_64".to_string(),
                guards: GuardConfig {
                    risk_threshold: "Low".to_string(),
                    min_health_threshold: 0.7,
                    max_cycle_latency_ms: 50,
                    repair_authority: "senior_engineer".to_string(),
                },
                policy: PolicyConfig {
                    profile: "strict_conformance".to_string(),
                    mdl_penalty: 0.05,
                    human_weight: 0.8,
                },
            },
            rl: RlConfig {
                algorithm: "DoubleQLearning".to_string(),
                learning_rate: 0.08,
                discount_factor: 0.95,
                exploration_rate: 0.2,
                exploration_decay: 0.999,
                reward_weights,
            },
            discovery: DiscoveryConfig {
                max_training_epochs: 100,
                fitness_stopping_threshold: 0.995,
                strategy: "incremental".to_string(),
                drift_window: 1000,
            },
            paths: PathConfig {
                training_logs_dir: "data/pdc2025/training_logs".to_string(),
                test_logs_dir: "data/pdc2025/test_logs".to_string(),
                ground_truth_dir: "data/pdc2025/ground_truth".to_string(),
                artifacts_dir: "artifacts".to_string(),
                manifest_bus_path: "tmp/dmanifest_bus".to_string(),
            },
            wasm: WasmConfig {
                batch_size: 10,
                max_pages: 16,
            },
            automl: AutomlConfig::default(),
        }
    }
}

impl AutonomicConfig {
    /// Compatibility loader: missing files use validated defaults.
    /// Call [`Self::load_required`] when the caller requires explicit configuration authority.
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Ok(Self::load_with_source(path)?.config)
    }

    /// Load configuration and retain whether authority came from a file or defaults.
    pub fn load_with_source<P: AsRef<Path>>(path: P) -> anyhow::Result<LoadedAutonomicConfig> {
        let path = path.as_ref();
        let loaded = if path.exists() {
            let content = fs::read_to_string(path)?;
            LoadedAutonomicConfig {
                config: toml::from_str(&content)?,
                source: ConfigSource::File(path.to_path_buf()),
            }
        } else {
            LoadedAutonomicConfig {
                config: Self::default(),
                source: ConfigSource::Default,
            }
        };
        loaded.config.validate()?;
        Ok(loaded)
    }

    /// Load only from an explicit file. Missing configuration is a typed error, never a default.
    pub fn load_required<P: AsRef<Path>>(path: P) -> anyhow::Result<LoadedAutonomicConfig> {
        let path = path.as_ref();
        if !path.exists() {
            anyhow::bail!("required dteam configuration is missing: {}", path.display());
        }
        Self::load_with_source(path)
    }

    /// Admit configuration only when numeric bounds and policy invariants are coherent.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        let mut violations = Vec::new();
        let finite_unit = |value: f32| value.is_finite() && (0.0..=1.0).contains(&value);

        if self.meta.version.trim().is_empty() {
            violations.push("meta.version must not be empty".to_string());
        }
        if self.meta.identity.trim().is_empty() {
            violations.push("meta.identity must not be empty".to_string());
        }
        if self.kernel.alignment == 0 || !self.kernel.alignment.is_power_of_two() {
            violations.push("kernel.alignment must be a non-zero power of two".to_string());
        }
        if !matches!(self.kernel.determinism.as_str(), "strict" | "seeded") {
            violations.push("kernel.determinism must be 'strict' or 'seeded'".to_string());
        }
        if !finite_unit(self.autonomic.guards.min_health_threshold) {
            violations.push("autonomic.guards.min_health_threshold must be in [0,1]".to_string());
        }
        if self.autonomic.guards.max_cycle_latency_ms == 0 {
            violations.push("autonomic.guards.max_cycle_latency_ms must be > 0".to_string());
        }
        for (name, value) in [
            ("rl.learning_rate", self.rl.learning_rate),
            ("rl.discount_factor", self.rl.discount_factor),
            ("rl.exploration_rate", self.rl.exploration_rate),
            ("rl.exploration_decay", self.rl.exploration_decay),
        ] {
            if !finite_unit(value) {
                violations.push(format!("{name} must be finite and in [0,1]"));
            }
        }
        if self.rl.reward_weights.is_empty() {
            violations.push("rl.reward_weights must not be empty".to_string());
        } else {
            let mut sum = 0.0_f32;
            for (name, value) in &self.rl.reward_weights {
                if !value.is_finite() || *value < 0.0 {
                    violations.push(format!("rl.reward_weights.{name} must be finite and >= 0"));
                }
                sum += *value;
            }
            if !sum.is_finite() || (sum - 1.0).abs() > 1e-5 {
                violations.push(format!("rl.reward_weights must sum to 1.0, observed {sum}"));
            }
        }
        if self.discovery.max_training_epochs == 0 {
            violations.push("discovery.max_training_epochs must be > 0".to_string());
        }
        if !self.discovery.fitness_stopping_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.discovery.fitness_stopping_threshold)
        {
            violations.push("discovery.fitness_stopping_threshold must be in [0,1]".to_string());
        }
        if self.discovery.drift_window == 0 {
            violations.push("discovery.drift_window must be > 0".to_string());
        }
        if self.wasm.batch_size == 0 {
            violations.push("wasm.batch_size must be > 0".to_string());
        }
        if self.wasm.max_pages == 0 {
            violations.push("wasm.max_pages must be > 0".to_string());
        }
        if self.automl.enabled && self.automl.budget == 0 {
            violations.push("automl.budget must be > 0 when AutoML is enabled".to_string());
        }
        if self.automl.successive_halving {
            if !self.automl.sh_subsample.is_finite()
                || !(0.0 < self.automl.sh_subsample && self.automl.sh_subsample <= 1.0)
            {
                violations.push("automl.sh_subsample must be in (0,1]".to_string());
            }
            if !self.automl.sh_promotion_ratio.is_finite()
                || self.automl.sh_promotion_ratio <= 1.0
            {
                violations.push("automl.sh_promotion_ratio must be > 1".to_string());
            }
        }
        for (name, value) in [
            ("paths.training_logs_dir", &self.paths.training_logs_dir),
            ("paths.test_logs_dir", &self.paths.test_logs_dir),
            ("paths.ground_truth_dir", &self.paths.ground_truth_dir),
            ("paths.artifacts_dir", &self.paths.artifacts_dir),
            ("paths.manifest_bus_path", &self.paths.manifest_bus_path),
        ] {
            if value.trim().is_empty() {
                violations.push(format!("{name} must not be empty"));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(ConfigValidationError { violations })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_admitted() {
        AutonomicConfig::default().validate().unwrap();
    }

    #[test]
    fn reward_weight_drift_is_refused() {
        let mut config = AutonomicConfig::default();
        config.rl.reward_weights.insert("fitness".to_string(), 0.9);
        let error = config.validate().unwrap_err();
        assert!(error.to_string().contains("must sum to 1.0"));
    }

    #[test]
    fn invalid_successive_halving_is_refused() {
        let mut config = AutonomicConfig::default();
        config.automl.successive_halving = true;
        config.automl.sh_subsample = 0.0;
        config.automl.sh_promotion_ratio = 1.0;
        let error = config.validate().unwrap_err();
        assert_eq!(error.violations().len(), 2);
    }

    #[test]
    fn missing_optional_config_reports_default_source() {
        let path = std::env::temp_dir().join("dteam-config-does-not-exist.toml");
        let loaded = AutonomicConfig::load_with_source(path).unwrap();
        assert_eq!(loaded.source, ConfigSource::Default);
    }

    #[test]
    fn missing_required_config_is_refused() {
        let path = std::env::temp_dir().join("dteam-required-config-does-not-exist.toml");
        let error = AutonomicConfig::load_required(path).unwrap_err();
        assert!(error.to_string().contains("required dteam configuration is missing"));
    }
}
