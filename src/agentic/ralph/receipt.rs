use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ReceiptEmitter;

impl Default for ReceiptEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceiptEmitter {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self, working_dir: &Path, idea: &str, final_hash: &str) -> Result<()> {
        let receipts_dir = working_dir.join(".receipts");
        fs::create_dir_all(&receipts_dir)?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let receipt_path = receipts_dir.join(format!("receipt_{}.txt", timestamp));

        let content = format!(
            "--- SPECKIT-RALPH RECEIPT ---\nTime: {}\nIdea: {}\nFinal Hash: {}\n\nCompleted successfully matching DfLSS/TPS gates and Ontology Closure.\n",
            timestamp, idea, final_hash
        );

        fs::write(receipt_path, content)?;
        Ok(())
    }
}
