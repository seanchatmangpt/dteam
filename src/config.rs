use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicConfig {
    pub rl: RlConfig,
    pub automation: AutomationConfig,
    pub paths: PathConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlConfig {
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub exploration_rate: f32,
    pub exploration_decay: f32,
    pub reinforce_learning_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub max_training_epochs: usize,
    pub fitness_stopping_threshold: f64,
    pub classification_fitness_threshold: f64,
    pub structural_soundness_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub training_logs_dir: String,
    pub test_logs_dir: String,
    pub ground_truth_dir: String,
}

impl Default for AutonomicConfig {
    fn default() -> Self {
        Self {
            rl: RlConfig {
                learning_rate: 0.1,
                discount_factor: 0.99,
                exploration_rate: 1.0,
                exploration_decay: 0.995,
                reinforce_learning_rate: 0.01,
            },
            automation: AutomationConfig {
                max_training_epochs: 10,
                fitness_stopping_threshold: 0.99,
                classification_fitness_threshold: 0.8,
                structural_soundness_penalty: 0.5,
            },
            paths: PathConfig {
                training_logs_dir: "training_logs".to_string(),
                test_logs_dir: "test_logs".to_string(),
                ground_truth_dir: "ground_truth".to_string(),
            },
        }
    }
}

impl AutonomicConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}
