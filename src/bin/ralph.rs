use dteam::agentic::ralph::{
    AgentKind, AutonomicController, ExecutionEngine, GitWorktreeManager, MaturityScorer,
    OntologyClosureEngine, PortfolioIndexer, RalphMode, ReceiptEmitter, SpecKitInvocation,
    SpecKitPhase, SpecKitRunner, SpeckitController, WorkSelector, WorkspaceManager,
};
use dteam::models::{Attribute, AttributeValue, Event, EventLog, Trace};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing::{error, info, info_span, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn init_telemetry() -> anyhow::Result<SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_attributes(vec![KeyValue::new("service.name", "ralph-orchestrator")])
                .build(),
        )
        .build();

    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("ralph-orchestrator");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();

    Ok(provider)
}

fn inject_supervisor(working_dir: &Path) -> anyhow::Result<()> {
    let gemini_dir = working_dir.join(".gemini");
    let hooks_dir = gemini_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    let settings = json!({
        "security": {
            "environmentVariableRedaction": { "enabled": true }
        },
        "hooks": {
            "BeforeTool": [
                {
                    "name": "ralph-supervisor",
                    "matcher": "write_file|replace|run_shell_command",
                    "type": "command",
                    "command": "./.gemini/hooks/supervisor.sh",
                    "timeout": 15000
                }
            ]
        }
    });

    fs::write(
        gemini_dir.join("settings.json"),
        serde_json::to_string_pretty(&settings)?,
    )?;

    let hook_script = r#"#!/usr/bin/env bash
input=$(cat)
tool_name=$(echo "$input" | jq -r '.tool_name')
tool_input=$(echo "$input" | jq -r '.tool_input')

if [[ "$tool_input" == *"AKIA"* || "$tool_input" == *"sk-ant"* || "$tool_input" == *"AIza"* ]]; then
    echo '{"decision": "deny", "reason": "SECURITY ALERT: Potential API Key detected in payload."}'
    exit 0
fi

if [[ "$tool_name" == "write_file" || "$tool_name" == "replace" ]]; then
    file_path=$(echo "$tool_input" | jq -r '.file_path')
    if [[ "$file_path" == *.rs ]]; then
        if [[ "$file_path" == *"lib.rs"* && "$tool_input" == *"syntax error"* ]]; then
             echo '{"decision": "deny", "reason": "SYNTAX VALIDATION: Prevented writing deliberate syntax error."}'
             exit 0
        fi
    fi
fi

echo '{"decision": "allow"}'
"#;

    let hook_path = hooks_dir.join("supervisor.sh");
    fs::write(&hook_path, hook_script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = init_telemetry().ok();
    let _main_span = info_span!("ralph_main").entered();

    if cfg!(debug_assertions) {
        info!("--- Ralph Wiggum Loop: Rust Parallel Orchestrator ---");
    }

    let root_dir = Path::new(".");
    let indexer = PortfolioIndexer::new();
    let state = indexer.scan(root_dir)?;
    info!(
        "Portfolio state scanned: {} active projects.",
        state.active_projects
    );

    let ontology = OntologyClosureEngine::new();
    let ontology_ctx = ontology.load_context(&root_dir.join("PUBLIC-ONTOLOGIES.ttl"))?;
    info!("Ontology loaded: {}", ontology_ctx);

    let scorer = MaturityScorer::new();
    let maturity = scorer.evaluate(&state)?;
    info!("Current Portfolio Maturity Level: {}", maturity);

    let selector = WorkSelector::new();
    let next_admissible_unit = selector.select_next(&state)?;
    info!("Next admissible unit: {}", next_admissible_unit);

    let args: Vec<String> = std::env::args().collect();
    let is_test = args.contains(&"--test".to_string());

    let mut max_concurrency = 1;
    if let Some(pos) = args.iter().position(|a| a == "--concurrency") {
        if let Some(val) = args.get(pos + 1) {
            max_concurrency = val.parse::<usize>().unwrap_or(1);
        }
    }

    let mut offset = 0;
    if let Some(pos) = args.iter().position(|a| a == "--offset") {
        if let Some(val) = args.get(pos + 1) {
            offset = val.parse::<usize>().unwrap_or(0);
        }
    }

    let mut limit = None;
    if let Some(pos) = args.iter().position(|a| a == "--limit") {
        if let Some(val) = args.get(pos + 1) {
            limit = val.parse::<usize>().ok();
        }
    }

    if is_test && cfg!(debug_assertions) {
        info!("!! TEST MODE ENABLED: Skipping LLM calls and using mock responses.");
    }

    let ideas_path = Path::new("IDEAS.md");
    if !ideas_path.exists() {
        fs::write(
            ideas_path,
            "1. Implement a basic health check endpoint\n2. Add logging to the autonomic cycle\n",
        )?;
    }

    let content = fs::read_to_string(ideas_path)?;
    let ideas: Vec<(String, String)> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, s)| (format!("{:03}", i + 1), s.to_string()))
        .skip(offset)
        .take(limit.unwrap_or(usize::MAX))
        .collect();

    let workspace_manager = GitWorktreeManager;
    workspace_manager.ensure_dev_branch()?;

    let meta_log = Arc::new(Mutex::new(EventLog::default()));
    let merge_lock = Arc::new(Mutex::new(()));

    let engine = ExecutionEngine::new(max_concurrency);

    let meta_log_clone = Arc::clone(&meta_log);
    let merge_lock_clone = Arc::clone(&merge_lock);

    let process_fn = move |id: String, idea: String| {
        let meta_log = Arc::clone(&meta_log_clone);
        let merge_lock = Arc::clone(&merge_lock_clone);

        async move {
            tokio::task::spawn_blocking(move || {
                let _span = info_span!("process_idea", idea = %idea, id = %id).entered();
                let slug = idea
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric(), "-")
                    .split('-')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("-");

                let working_dir = PathBuf::from(".wreckit").join(format!("{}-{}", id, slug));
                fs::create_dir_all(&working_dir)?;

                if cfg!(debug_assertions) {
                    info!("\n[Idea {}] Processing: {}", id, idea);
                }

                let mut trace = Trace {
                    id: id.clone(),
                    ..Default::default()
                };

                let branch_name = format!("wreckit/{}", slug);
                let worktree_path = working_dir.join("worktree");

                let workspace = GitWorktreeManager;
                if let Err(e) = workspace.setup_worktree(&branch_name, &worktree_path) {
                    error!("Failed to setup worktree: {}", e);
                    return Ok::<(), anyhow::Error>(());
                }

                let controller = SpeckitController::new();
                let start = Instant::now();

                let root_path = std::env::current_dir().unwrap_or(PathBuf::from("."));
                let script_path = root_path.join("scripts").join("mcp_plus_dogfood_loop.sh");

                let invocation = SpecKitInvocation {
                    phase: SpecKitPhase::Implement,
                    mode: RalphMode::Exploit,
                    agent: AgentKind::ClaudeCode,
                    command: format!("{} --target \"{}\"", script_path.display(), slug),
                    working_dir: worktree_path.clone(),
                    may_write: true,
                };

                let phase_name = format!("{:?}", invocation.phase);
                let _phase_span =
                    info_span!("run_phase", phase = %phase_name, idea = %idea).entered();

                if is_test {
                    let _ = std::fs::write(working_dir.join("research.md"), "MOCK");
                    let _ = std::fs::write(working_dir.join("plan.md"), "MOCK");
                    let _ = std::fs::write(working_dir.join("implement.md"), "MOCK");
                } else {
                    match controller.invoke(invocation) {
                        Ok(receipt) => {
                            if !receipt.success {
                                error!("Phase {} failed: {}", phase_name, receipt.output);
                                let _ = workspace.cleanup_worktree(&worktree_path);
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            error!("Phase {} execution error: {}", phase_name, e);
                            let _ = workspace.cleanup_worktree(&worktree_path);
                            return Ok(());
                        }
                    }
                }

                let mut event = Event::new(phase_name);
                event.attributes.push(Attribute {
                    key: "idea".to_string(),
                    value: AttributeValue::String(idea.clone()),
                });
                event.attributes.push(Attribute {
                    key: "duration_ns".to_string(),
                    value: AttributeValue::String(start.elapsed().as_nanos().to_string()),
                });
                trace.events.push(event);

                if let Err(e) = inject_supervisor(&worktree_path) {
                    error!("Failed to inject supervisor: {}", e);
                }

                if let Err(e) = workspace.commit_changes(&worktree_path, &id, &idea) {
                    error!("Failed to commit changes: {}", e);
                }

                {
                    let _lock = merge_lock.lock().unwrap();
                    if let Err(e) = workspace.merge_into_dev(&branch_name) {
                        warn!("Failed to merge branch {}: {}", branch_name, e);
                    }
                }

                let _ = workspace.cleanup_worktree(&worktree_path);

                let receipt_emitter = ReceiptEmitter::new();
                if let Err(e) = receipt_emitter.emit(&working_dir, &idea, "HashVerified") {
                    error!("Failed to emit receipt: {}", e);
                }

                meta_log.lock().unwrap().add_trace(trace);

                Ok::<(), anyhow::Error>(())
            })
            .await?
        }
    };

    engine.run(ideas, process_fn).await?;

    let final_log = meta_log.lock().unwrap().clone();
    let controller = AutonomicController::new("IDEAS.md");
    if let Err(e) = controller.evaluate_dogfood(&final_log) {
        error!("Autonomic cycle failed: {}", e);
    }

    if cfg!(debug_assertions) {
        info!("\n--- All ideas processed! ---");
    }

    if let Some(p) = provider {
        let _ = p.force_flush();
    }

    Ok(())
}
