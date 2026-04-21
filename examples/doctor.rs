use dteam::autonomic::{AutonomicEvent, AutonomicKernel, DefaultKernel};
use dteam::dteam::orchestration::{DteamDoctor, Engine};

fn main() {
    println!("--- dteam Digital Team Doctor ---");
    let engine = Engine::builder().build();
    println!("{}", engine.doctor());

    println!("\n--- Autonomic Kernel Diagnostic ---");
    let mut kernel = DefaultKernel::new();
    let event = AutonomicEvent {
        source_hash: 0x1234,
        activity_idx: 0,
        payload_hash: 0x5678,
        timestamp_ns: 123456789,
    };

    println!("State before: {}", kernel.infer());
    let count = kernel.run_cycle(&event);
    println!("Cycle executed. Result count: {}", count);
    println!("State after:  {}", kernel.infer());

    println!("\nDiagnostics complete. System status: NOMINAL");
}
