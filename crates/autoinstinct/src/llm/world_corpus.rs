//! Bridge: OcelWorld → TraceCorpus
//!
//! Convert an admitted OCEL world into a trace corpus ready for motif discovery.
//! Each OcelEvent maps to an Episode with context URN derived from event attributes
//! + object set, response from expectedResponse, and receipt URN from event hash.

use std::collections::BTreeMap;

use crate::corpus::{Episode, TraceCorpus};
use ccog::instinct::AutonomicInstinct;

use super::schema::{Counterfactual, OcelEvent, OcelObject, OcelWorld};
use super::LlmAdmissionError;

/// Convert an OCEL world into a trace corpus.
///
/// # Errors
///
/// Returns `LlmAdmissionError::Structural` if the world references
/// objects that don't exist in the declared object set.
pub fn world_to_corpus(world: &OcelWorld) -> Result<TraceCorpus, LlmAdmissionError> {
    // Validate that counterfactuals is not empty (already checked at admission,
    // but re-validate here for defense in depth).
    if world.counterfactuals.is_empty() {
        return Err(LlmAdmissionError::Structural(
            "world has zero counterfactuals; cannot convert to corpus".into(),
        ));
    }

    // Build object ID set for validation.
    let object_ids: std::collections::HashSet<_> = world
        .objects
        .iter()
        .map(|o| o.id.as_str())
        .collect();

    let mut corpus = TraceCorpus::new();

    for event in &world.events {
        // Validate that all object references exist.
        for obj_id in &event.objects {
            if !object_ids.contains(obj_id.as_str()) {
                return Err(LlmAdmissionError::Structural(format!(
                    "event {} references undefined object id: {}",
                    event.id, obj_id
                )));
            }
        }

        // Derive context URN from event ID + object set + attributes.
        let context_urn = derive_context_urn(event, &object_ids);

        // Use event's expected_response if present, otherwise default to Ask.
        let response = event
            .expected_response
            .clone()
            .unwrap_or(AutonomicInstinct::Ask);

        // Derive receipt URN from event hash.
        let receipt_urn = derive_receipt_urn(event);

        corpus.push(Episode {
            context_urn,
            response,
            receipt_urn,
            outcome: event.outcome.clone(),
        });
    }

    Ok(corpus)
}

/// Derive context URN from event ID, object set, and attributes.
/// Concatenate in canonical form and hash with blake3.
fn derive_context_urn(event: &OcelEvent, _object_ids: &std::collections::HashSet<&str>) -> String {
    // Canonical form: event_id || attribute_json || object_ids sorted
    let attr_json = serde_json::to_string(&event.attributes)
        .unwrap_or_else(|_| "{}".to_string());

    let mut obj_ids = event.objects.clone();
    obj_ids.sort();

    let material = format!(
        "{}\x00{}\x00{}",
        event.id,
        attr_json,
        obj_ids.join("\x00")
    );

    let hash = blake3::hash(material.as_bytes());
    format!("urn:blake3:{}", hash.to_hex())
}

/// Derive receipt URN from event by hashing its JSON serialization.
fn derive_receipt_urn(event: &OcelEvent) -> String {
    let json = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
    let hash = blake3::hash(json.as_bytes());
    format!("urn:blake3:{}", hash.to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_corpus_rejects_empty_counterfactuals() {
        let world = OcelWorld {
            version: "1.0".into(),
            profile: "core".into(),
            scenario: "test".into(),
            objects: vec![OcelObject {
                id: "obj1".into(),
                kind: "entity".into(),
                label: "Test Object".into(),
                ontology_type: "https://schema.org/Thing".into(),
                attributes: BTreeMap::new(),
            }],
            events: vec![],
            counterfactuals: vec![],
            expected_instincts: vec![],
        };

        let result = world_to_corpus(&world);
        assert!(matches!(
            result,
            Err(LlmAdmissionError::Structural(_))
        ));
    }

    #[test]
    fn world_to_corpus_rejects_dangling_object_reference() {
        let world = OcelWorld {
            version: "1.0".into(),
            profile: "core".into(),
            scenario: "test".into(),
            objects: vec![OcelObject {
                id: "obj1".into(),
                kind: "entity".into(),
                label: "Test Object".into(),
                ontology_type: "https://schema.org/Thing".into(),
                attributes: BTreeMap::new(),
            }],
            events: vec![OcelEvent {
                id: "evt1".into(),
                kind: "action".into(),
                time: "2026-04-30T12:00:00Z".into(),
                objects: vec!["obj_nonexistent".into()],
                ontology_type: "https://schema.org/Action".into(),
                attributes: BTreeMap::new(),
                expected_response: Some(AutonomicInstinct::Ask),
                outcome: None,
            }],
            counterfactuals: vec![Counterfactual {
                id: "cf1".into(),
                description: "test".into(),
                remove_objects: vec![],
                remove_events: vec![],
                expected_response: AutonomicInstinct::Ask,
            }],
            expected_instincts: vec![],
        };

        let result = world_to_corpus(&world);
        assert!(matches!(
            result,
            Err(LlmAdmissionError::Structural(_))
        ));
    }

    #[test]
    fn world_to_corpus_produces_episodes_with_derived_urns() {
        let world = OcelWorld {
            version: "1.0".into(),
            profile: "core".into(),
            scenario: "test".into(),
            objects: vec![OcelObject {
                id: "obj1".into(),
                kind: "entity".into(),
                label: "Test Object".into(),
                ontology_type: "https://schema.org/Thing".into(),
                attributes: BTreeMap::new(),
            }],
            events: vec![OcelEvent {
                id: "evt1".into(),
                kind: "action".into(),
                time: "2026-04-30T12:00:00Z".into(),
                objects: vec!["obj1".into()],
                ontology_type: "https://schema.org/Action".into(),
                attributes: {
                    let mut map = BTreeMap::new();
                    map.insert("status".to_string(), serde_json::json!("success"));
                    map
                },
                expected_response: Some(AutonomicInstinct::Settle),
                outcome: Some("earned".into()),
            }],
            counterfactuals: vec![Counterfactual {
                id: "cf1".into(),
                description: "test".into(),
                remove_objects: vec![],
                remove_events: vec![],
                expected_response: AutonomicInstinct::Ask,
            }],
            expected_instincts: vec![],
        };

        let result = world_to_corpus(&world);
        assert!(result.is_ok());

        let corpus = result.unwrap();
        assert_eq!(corpus.len(), 1);

        let ep = &corpus.episodes[0];
        assert_eq!(ep.response, AutonomicInstinct::Settle);
        assert!(ep.context_urn.starts_with("urn:blake3:"));
        assert!(ep.receipt_urn.starts_with("urn:blake3:"));
        assert_eq!(ep.outcome, Some("earned".into()));
    }
}
