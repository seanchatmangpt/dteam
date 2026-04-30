//! Multimodal posture and context bundles (Phase 5 Track F stub).
//!
//! `PostureBundle` captures interpreter posture (audio cadence, gaze
//! orientation, body tension, settling). `ContextBundle` captures local
//! cognition surface — expectation, risk, affordance. The `Track F` writer
//! fleshes out ingest helpers and bit-name constants.

/// Bit positions for canonical posture predicates.
#[allow(non_snake_case)]
pub mod PostureBit {
    /// Subject is calm — no current alert.
    pub const CALM: u32 = 0;
    /// Subject has detected a single signal.
    pub const ALERT: u32 = 1;
    /// Subject has multiple corroborating signals.
    pub const ENGAGED: u32 = 2;
    /// Subject has resolved its stance, returned to baseline.
    pub const SETTLED: u32 = 3;
    /// Subject orientation toward an entry/door.
    pub const ORIENTED_TO_ENTRY: u32 = 4;
    /// Subject orientation toward an internal source.
    pub const ORIENTED_INTERIOR: u32 = 5;
    /// Cadence indicates a known package/delivery class.
    pub const CADENCE_DELIVERY: u32 = 6;
    /// Cadence indicates a known wife/partner arrival class.
    pub const CADENCE_PARTNER: u32 = 7;
}

/// Bit positions for canonical context predicates.
#[allow(non_snake_case)]
pub mod ContextBit {
    /// A package is currently expected.
    pub const PACKAGE_EXPECTED: u32 = 0;
    /// Partner arrival is currently expected.
    pub const PARTNER_DUE: u32 = 1;
    /// Maintenance is currently scheduled.
    pub const MAINTENANCE_SCHEDULED: u32 = 2;
    /// Theft risk is currently elevated.
    pub const THEFT_RISK: u32 = 3;
    /// Safety risk is currently elevated.
    pub const SAFETY_RISK: u32 = 4;
    /// Retrieval is currently low-cost / available.
    pub const CAN_RETRIEVE_NOW: u32 = 5;
    /// Inspection is currently feasible.
    pub const CAN_INSPECT: u32 = 6;
    /// Escalation is mandatory.
    pub const MUST_ESCALATE: u32 = 7;
}

/// Multimodal posture bundle from the trusted local interpreter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PostureBundle {
    /// Bitmask of [`PostureBit`] entries set by the interpreter.
    pub posture_mask: u64,
    /// Confidence in the posture interpretation (0-255).
    pub confidence: u8,
}

/// Local cognition context — expectation, risk, affordance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContextBundle {
    /// Bits from [`ContextBit`] indicating expected events.
    pub expectation_mask: u64,
    /// Bits from [`ContextBit`] indicating elevated risks.
    pub risk_mask: u64,
    /// Bits from [`ContextBit`] indicating available affordances.
    pub affordance_mask: u64,
}

impl PostureBundle {
    /// True iff `bit` is set in the posture mask.
    #[must_use]
    pub const fn has(&self, bit: u32) -> bool {
        (self.posture_mask >> bit) & 1 == 1
    }
}

impl ContextBundle {
    /// True iff `bit` is set in any of expectation/risk/affordance masks.
    #[must_use]
    pub const fn has_any(&self, bit: u32) -> bool {
        let m = 1u64 << bit;
        (self.expectation_mask & m) != 0
            || (self.risk_mask & m) != 0
            || (self.affordance_mask & m) != 0
    }
}
