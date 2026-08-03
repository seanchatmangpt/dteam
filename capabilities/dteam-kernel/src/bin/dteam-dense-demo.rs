//! End-to-end demonstration across access, transport, quota, state, and saga capabilities.

use dteam_capability_kernel::access::{
    AccessEffect, AccessGrant, AccessPolicy, AccessPrincipalId, AccessResourceId, GrantId,
    PermissionId, ResourceScope, RoleAssignment, RoleId,
};
use dteam_capability_kernel::event_bus::{
    ConsumerId, EventBus, EventId, PublishRequest, Subscription, SubscriptionId, TopicConfig,
    TopicId,
};
use dteam_capability_kernel::model::{CapabilityId, Observation, OperationId, SubjectId};
use dteam_capability_kernel::quota::{
    PrincipalId, QuotaClaim, QuotaManager, QuotaPolicy, ReservationId, ReservationRequest,
    ResourceId,
};
use dteam_capability_kernel::saga::{
    SagaCoordinator, SagaDefinition, SagaDefinitionId, SagaExecutor, SagaId, SagaOperation,
    SagaOperationResult, SagaStep, SagaStepId,
};
use dteam_capability_kernel::store::{
    ExpectedVersion, Mutation, RecordKey, Transaction, TransactionalStore,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut access = AccessPolicy::new();
    let operator = RoleId::new("operator")?;
    access.add_role(operator.clone())?;
    access.add_grant(AccessGrant::new(
        GrantId::new("operate-cases")?,
        operator.clone(),
        PermissionId::new("execute")?,
        ResourceScope::new("/cases")?,
        AccessEffect::Allow,
    ))?;
    access.assign(RoleAssignment::new(
        AccessPrincipalId::new("worker-7")?,
        operator,
        0,
        None,
    )?)?;
    let context = Observation::new(SubjectId::new("request-1")?, 1);
    let access_decision = access.evaluate(
        &AccessPrincipalId::new("worker-7")?,
        &PermissionId::new("execute")?,
        &AccessResourceId::new("/cases/42")?,
        1,
        &context,
    );
    if !access_decision.allowed() {
        return Err("access was not admitted".into());
    }

    let mut quota = QuotaManager::new();
    quota.set_policy(
        PrincipalId::new("worker-7")?,
        ResourceId::new("execution-slot")?,
        QuotaPolicy::new(8, 1, 10, 2, 30)?,
        0,
    )?;
    let reservation_id = ReservationId::new("case-42-run")?;
    quota.reserve(ReservationRequest::new(
        reservation_id.clone(),
        1,
        vec![QuotaClaim::new(
            PrincipalId::new("worker-7")?,
            ResourceId::new("execution-slot")?,
            1,
        )?],
    )?)?;

    let mut bus = EventBus::new();
    let topic = TopicId::new("case-events")?;
    bus.create_topic(topic.clone(), TopicConfig::new(4, 1_024, 10, 3)?)?;
    let subscription_id = SubscriptionId::new("case-projector")?;
    let consumer = ConsumerId::new("projector-1")?;
    bus.subscribe(Subscription::new(
        subscription_id.clone(),
        topic.clone(),
        consumer.clone(),
    ))?;
    let published = bus
        .publish(PublishRequest::new(
            EventId::new("case-42-opened")?,
            topic,
            2,
            b"case-42".to_vec(),
            b"opened".to_vec(),
        ))?
        .clone();
    let delivery = bus
        .poll(&subscription_id, &consumer, 3, 1)?
        .into_iter()
        .next()
        .ok_or("event was not delivered")?;
    bus.acknowledge(&subscription_id, delivery.id(), &consumer, 4)?;

    let mut store = TransactionalStore::new();
    let case_key = RecordKey::new("cases/42/status")?;
    store.commit(Transaction::new(
        "project-case-42",
        4,
        vec![Mutation::Put {
            key: case_key.clone(),
            value: published.payload().to_vec(),
            expected: ExpectedVersion::Missing,
            expires_at: None,
        }],
    )?)?;

    let definition = SagaDefinition::new(
        SagaDefinitionId::new("case-execution")?,
        1,
        vec![
            SagaStep::new(
                SagaStepId::new("prepare")?,
                SagaOperation::new(
                    CapabilityId::new("case-runtime")?,
                    OperationId::new("prepare")?,
                    Vec::new(),
                ),
            )
            .compensate_with(SagaOperation::new(
                CapabilityId::new("case-runtime")?,
                OperationId::new("unprepare")?,
                Vec::new(),
            )),
            SagaStep::new(
                SagaStepId::new("execute")?,
                SagaOperation::new(
                    CapabilityId::new("case-runtime")?,
                    OperationId::new("execute")?,
                    Vec::new(),
                ),
            )
            .compensate_with(SagaOperation::new(
                CapabilityId::new("case-runtime")?,
                OperationId::new("rollback")?,
                Vec::new(),
            )),
        ],
    )?;
    let saga_id = SagaId::new("case-42")?;
    let mut coordinator = SagaCoordinator::new();
    coordinator.start(saga_id.clone(), &definition, b"case-42".to_vec())?;
    let mut executor = DenseExecutor;
    coordinator.run(&saga_id, &definition, &mut executor)?;
    let saga_verification = coordinator.verify(&saga_id, &definition)?;

    quota.commit(&reservation_id, 5)?;
    let quota_verification = quota.verify()?;
    let store_verification = store.verify()?;

    println!(
        concat!(
            "{{",
            "\"standing\":\"ALIVE\",",
            "\"access\":\"{}\",",
            "\"event\":\"{}\",",
            "\"event_bus_state\":\"{}\",",
            "\"quota_head\":\"{}\",",
            "\"quota_receipts\":{},",
            "\"store_state\":\"{}\",",
            "\"store_commits\":{},",
            "\"saga_state\":\"{}\",",
            "\"saga_head\":\"{}\",",
            "\"projected_status\":\"{}\"",
            "}}"
        ),
        access_decision.digest(),
        published.digest(),
        bus.state_digest(),
        quota_verification.head(),
        quota_verification.receipts(),
        store_verification.state_digest(),
        store_verification.commits(),
        saga_verification.state().as_str(),
        saga_verification.head(),
        String::from_utf8_lossy(
            store
                .get(&case_key)
                .ok_or("projected state is missing")?
                .value()
        ),
    );
    Ok(())
}

struct DenseExecutor;

impl SagaExecutor for DenseExecutor {
    fn execute(
        &mut self,
        _saga: &SagaId,
        step: &SagaStep,
        _attempt: u32,
        input: &[u8],
    ) -> SagaOperationResult {
        let mut output = input.to_vec();
        output.extend_from_slice(b":");
        output.extend_from_slice(step.id().as_str().as_bytes());
        SagaOperationResult::Succeeded { output, elapsed: 1 }
    }

    fn compensate(
        &mut self,
        _saga: &SagaId,
        _step: &SagaStep,
        _attempt: u32,
        _forward_output: &[u8],
    ) -> SagaOperationResult {
        SagaOperationResult::Succeeded {
            output: Vec::new(),
            elapsed: 1,
        }
    }
}
