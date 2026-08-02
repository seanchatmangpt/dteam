use dteam_capability_kernel::{
    standard_combinatorial_engine, FeatureId, InnovationAudit, QolCatalog, ServiceObjective,
    TelcoTopology, Vision2030, VisionWizard, WizardValue,
};

fn print_help() {
    println!("dteam-doctor [--json|status|repair|graph|qol <profile>|wizard [preset]|compose [preset]|telco|innovation|innovation-json|snapshot|support|crown]");
    println!("presets: developer | edge | telco | enterprise");
}

fn wizard_for(preset: &str) -> VisionWizard {
    let mut wizard = VisionWizard::standard();
    let answers = match preset {
        "edge" => [("mode", "edge"), ("availability", "ha"), ("authority", "yes"), ("offline", "yes"), ("reversible", "yes")],
        "telco" => [("mode", "telco"), ("availability", "carrier"), ("authority", "yes"), ("offline", "yes"), ("reversible", "yes")],
        "enterprise" => [("mode", "enterprise"), ("availability", "ha"), ("authority", "yes"), ("offline", "no"), ("reversible", "no")],
        _ => [("mode", "developer"), ("availability", "standard"), ("authority", "yes"), ("offline", "yes"), ("reversible", "yes")],
    };
    for (id, value) in answers {
        wizard.answer(id, WizardValue::Choice(value.to_owned())).expect("static wizard answer");
    }
    wizard
}

fn print_compositions(preset: &str) {
    let wizard = wizard_for(preset);
    let plan = wizard.compile().expect("complete preset");
    let engine = standard_combinatorial_engine();
    let space = engine.explore(plan.request()).expect("bounded composition search");
    println!(
        "preset={} explored={} refused={} lawful={} pareto={} plan={} space={}",
        preset,
        space.explored(),
        space.refused(),
        space.lawful().len(),
        space.pareto().len(),
        plan.digest(),
        space.digest()
    );
    for (index, composition) in space.pareto().iter().enumerate() {
        println!(
            "{}. cost={} latency_us={} reliability_ppm={} complexity={} reversible={} digest={} components=[{}]",
            index + 1,
            composition.cost(),
            composition.latency_micros(),
            composition.reliability_ppm(),
            composition.complexity(),
            composition.is_reversible(),
            composition.digest(),
            composition.components().join(",")
        );
    }
}

fn print_telco() {
    let topology = TelcoTopology::standard();
    let objective = ServiceObjective {
        maximum_latency_micros: 2_000,
        minimum_capacity: 8_000,
        minimum_reliability_ppm: 999_999,
        minimum_failure_domains: 3,
    };
    let assessment = topology.assess("edge-a", "core-a", &objective);
    println!(
        "TELCO standing={} paths={} disjoint={} spof={} digest={}",
        assessment.standing(),
        assessment.compliant_paths().len(),
        assessment.disjoint_path_count(),
        assessment.single_points_of_failure().len(),
        assessment.digest()
    );
    for path in assessment.compliant_paths() {
        println!(
            "path=[{}] latency_us={} capacity={} reliability_ppm={} domains={} digest={}",
            path.nodes().join("->"),
            path.latency_micros(),
            path.capacity(),
            path.reliability_ppm(),
            path.failure_domains().len(),
            path.digest()
        );
    }
    for node in assessment.single_points_of_failure() { println!("spof:{node}"); }
    if assessment.standing() == "BLOCKED" { std::process::exit(4); }
}

fn print_innovation_json() {
    let audit = InnovationAudit::run();
    println!("{}", audit.to_json());
    if !audit.standing().is_usable() {
        std::process::exit(5);
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let vision = Vision2030::standard();
    let report = vision.diagnose();
    match args.first().map(String::as_str).unwrap_or("status") {
        "--json" | "status" => println!("{}", report.to_json()),
        "repair" => {
            let plan = vision.repair_plan();
            println!("standing={} current_score={} projected_score_if_all_actions_succeed={} plan={}", report.standing(), report.score(), plan.projected_score(), plan.digest());
            for (index, action) in plan.actions().iter().enumerate() {
                println!("{}. [{} impact={} reversible={}] {} :: {}", index + 1, action.capability(), action.impact(), action.reversible(), action.command(), action.reason());
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
        "wizard" => {
            let preset = args.get(1).map(String::as_str).unwrap_or("developer");
            let wizard = wizard_for(preset);
            let plan = wizard.compile().expect("preset is complete");
            println!("WIZARD preset={} digest={}", preset, plan.digest());
            for command in plan.commands() { println!("{command}"); }
        }
        "compose" => {
            let preset = args.get(1).map(String::as_str).unwrap_or("developer");
            print_compositions(preset);
        }
        "telco" => print_telco(),
        "innovation" => {
            let audit = InnovationAudit::run();
            print!("{}", audit.to_markdown());
            if !audit.standing().is_usable() {
                std::process::exit(5);
            }
        }
        "innovation-json" => print_innovation_json(),
        "snapshot" => {
            let audit = InnovationAudit::run();
            println!("{}", audit.snapshot().to_json());
            if !audit.standing().is_usable() {
                std::process::exit(5);
            }
        }
        "support" => {
            let audit = InnovationAudit::run();
            let bundle = audit.support_bundle();
            println!("{}", bundle.to_json());
            if !bundle.verify() || !audit.standing().is_usable() {
                std::process::exit(5);
            }
        }
        "feature" => {
            let Some(name) = args.get(1) else { eprintln!("feature name required"); std::process::exit(2); };
            match FeatureId::new(name.clone()) {
                Ok(feature) => println!("feature:{}", feature),
                Err(error) => { eprintln!("{error}"); std::process::exit(2); }
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
