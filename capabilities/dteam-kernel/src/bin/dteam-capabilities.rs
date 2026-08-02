use dteam_capability_kernel::prelude::*;
use dteam_capability_kernel::{AdmissionDecision, ProcessResult};

struct DemoExecutor;

impl Executor for DemoExecutor {
    fn id(&self) -> &str {
        "demo-executor"
    }

    fn preflight(&self, intent: &Intent) -> Result<(), PreflightRefusal> {
        if intent.payload().is_empty() {
            Err(PreflightRefusal {
                code: "EMPTY_PAYLOAD".to_owned(),
                detail: "notification payload must not be empty".to_owned(),
            })
        } else {
            Ok(())
        }
    }

    fn execute(&mut self, intent: &Intent) -> Outcome {
        Outcome::Applied {
            code: 200,
            output: intent.payload().to_vec(),
        }
    }
}

fn build_runtime() -> Runtime {
    let capability = CapabilityId::new("notification.delivery").expect("static capability id");
    let operation = OperationId::new("send").expect("static operation id");
    let authority = AuthorityId::new("operations").expect("static authority id");

    let mut graph = CapabilityGraph::new();
    graph
        .insert(
            Capability::new(CapabilityId::new("observation.admission").unwrap())
                .supports(OperationId::new("admit").unwrap())
                .cost_units(1),
        )
        .expect("unique capability");
    graph
        .insert(
            Capability::new(capability.clone())
                .depends_on(CapabilityId::new("observation.admission").unwrap())
                .supports(operation.clone())
                .allows(authority)
                .reversible(false)
                .cost_units(3),
        )
        .expect("unique capability");

    let mut router = Router::new();
    router.insert(Route::new("notifications.send", capability, operation));

    let policy = AdmissionPolicy::new(PolicyId::new("notification-policy").unwrap(), 1)
        .with_rule(Rule::new(
            "ready",
            Predicate::Equals {
                key: "ready".to_owned(),
                expected: true.into(),
            },
        ))
        .with_rule(Rule::new(
            "active-risk-window",
            Predicate::MaxU64 {
                key: "risk".to_owned(),
                maximum: 2,
            },
        ));

    Runtime::new(router, policy, Broker::new("brce", graph, 10).unwrap())
}

fn build_process_log() -> ObjectEventLog {
    let mut log = ObjectEventLog::new();
    let case = ObjectId::new("case-42").unwrap();
    log.add_object(ObjectRecord::new(
        case.clone(),
        ObjectType::new("case").unwrap(),
    ))
    .unwrap();
    for (index, activity) in ["observe", "admit", "construct", "actuate", "receipt"]
        .into_iter()
        .enumerate()
    {
        log.append_event(
            EventRecord::new(
                EventId::new(format!("event-{index}")).unwrap(),
                Activity::new(activity).unwrap(),
                index as u64,
            )
            .relating(case.clone()),
        )
        .unwrap();
    }
    log
}

fn emit_json(result: &ProcessResult, runtime: &Runtime, log: &ObjectEventLog) {
    let model = discover_transition_system(log, 1);
    let conformance = model.conform(log);
    let verification = runtime.broker().verify().expect("broker replay");
    println!(
        concat!(
            "{{\n",
            "  \"standing\": \"{:?}\",\n",
            "  \"intent\": \"{}\",\n",
            "  \"completion_receipt\": \"{}\",\n",
            "  \"trace_head\": \"{}\",\n",
            "  \"authorization_head\": \"{}\",\n",
            "  \"completion_head\": \"{}\",\n",
            "  \"process_log\": \"{}\",\n",
            "  \"process_model\": \"{}\",\n",
            "  \"fitness\": {:.6},\n",
            "  \"conformant\": {}\n",
            "}}"
        ),
        verification.standing(),
        result.intent().digest(),
        result.evidence().completion().digest(),
        result.trace().head(),
        verification.authorization_head(),
        verification.completion_head(),
        log.digest(),
        model.digest(),
        conformance.fitness(),
        conformance.is_conformant(),
    );
}

fn main() {
    let mut runtime = build_runtime();
    let mut executor = DemoExecutor;
    let mut observation = Observation::new(SubjectId::new("case-42").unwrap(), 7);
    observation.insert("ready", true).unwrap();
    observation.insert("risk", 1_u64).unwrap();
    observation
        .attest(AuthorityId::new("operations").unwrap());

    let result = runtime
        .process(
            &mut executor,
            "notifications.send",
            observation,
            AuthorityId::new("operations").unwrap(),
            1,
            b"release approved".to_vec(),
        )
        .unwrap_or_else(|error| panic!("runtime failed: {error}"));
    assert!(matches!(
        runtime.policy().evaluate(
            Observation::new(SubjectId::new("dry-run").unwrap(), 1)
        ),
        AdmissionDecision::Refused(_)
    ));
    let log = build_process_log();
    emit_json(&result, &runtime, &log);
}
