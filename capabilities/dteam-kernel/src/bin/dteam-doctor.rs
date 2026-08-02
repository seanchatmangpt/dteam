use dteam_capability_kernel::{QolCatalog, Vision2030};

fn print_help() {
    println!("dteam-doctor [--json|status|repair|graph|qol <profile>|crown]");
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let vision = Vision2030::standard();
    let report = vision.diagnose();
    match args.first().map(String::as_str).unwrap_or("status") {
        "--json" | "status" => println!("{}", report.to_json()),
        "repair" => {
            let plan = vision.repair_plan();
            println!("standing={} current_score={} projected_score={} plan={}", report.standing(), report.score(), plan.projected_score(), plan.digest());
            for (index, action) in plan.actions().iter().enumerate() {
                println!("{}. [{} impact={}] {}", index + 1, action.capability(), action.impact(), action.command());
            }
        }
        "graph" => {
            for id in vision.topological_order() {
                let capability = vision.capability(&id).expect("ordered capability exists");
                let dependencies = capability.dependencies().iter().cloned().collect::<Vec<_>>().join(",");
                println!("{} [{}] <- [{}] :: {}", capability.id(), capability.standing_value(), dependencies, capability.proof_command());
            }
        }
        "qol" => {
            let catalog = QolCatalog::standard();
            let Some(name) = args.get(1) else {
                for profile in catalog.profiles().values() {
                    println!("{}: {}", profile.name(), profile.description());
                }
                return;
            };
            match catalog.profile(name) {
                Some(profile) => {
                    println!("# {} — {}", profile.name(), profile.description());
                    for command in profile.commands() { println!("{command}"); }
                }
                None => {
                    eprintln!("unknown QoL profile `{name}`");
                    std::process::exit(2);
                }
            }
        }
        "crown" => {
            println!("VISION_2030 standing={} score={} digest={}", report.standing(), report.score(), report.digest());
            for item in report.critical_path() { println!("critical:{item}"); }
            if report.score() < 100 { std::process::exit(3); }
        }
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("unknown command `{other}`");
            print_help();
            std::process::exit(2);
        }
    }
}
