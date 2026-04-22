/**
 * TypeScript mirror of unibit_motion_tick semantics.
 *
 * Pure TS, no napi-rs yet — this file reproduces the branchless
 * admission algebra so the visualizer can run standalone. When the
 * N-API bridge is wired, swap this implementation for the real call
 * without changing any callers.
 */

// =============================================================================
// Types
// =============================================================================

export type FieldLaneName =
  | 'prereq'
  | 'capability'
  | 'causality'
  | 'conformance'
  | 'law'
  | 'scenario'
  | 'risk'
  | 'attention';

export const LANES: FieldLaneName[] = [
  'prereq',
  'capability',
  'causality',
  'conformance',
  'law',
  'scenario',
  'risk',
  'attention',
];

export const LANE_IS_REQUIRED: Record<FieldLaneName, boolean> = {
  prereq: true,
  capability: true,
  causality: true,
  conformance: true,
  law: false,
  scenario: false,
  risk: false,
  attention: false,
};

/** Lane hue palette from doc 60. */
export const LANE_HUE: Record<FieldLaneName, string> = {
  prereq: '#4db2ff',
  capability: '#e68019',
  causality: '#f2f24a',
  conformance: '#b3b3b3',
  law: '#e63333',
  scenario: '#9a1acc',
  risk: '#33cc4d',
  attention: '#ff66c4',
};

export interface PackedEightField {
  prereq_required: bigint;
  capability_required: bigint;
  causality_required: bigint;
  conformance_required: bigint;
  law_forbidden: bigint;
  scenario_forbidden: bigint;
  risk_forbidden: bigint;
  attention_forbidden: bigint;
}

export interface MotionRequest {
  state: bigint;
  fields: PackedEightField;
  instructionId: bigint;
}

export interface MotionResponse {
  nextMarking: bigint;
  denyTotal: bigint;
  fragment: bigint;
  status: number;
  perLane: Record<FieldLaneName, bigint>;
}

// =============================================================================
// Branchless admission — mirrors unibit-hot/t0 admit8_t0 exactly.
// =============================================================================

function denyRequired(state: bigint, required: bigint): bigint {
  // missing_required = (state & required) XOR required
  return (state & required) ^ required;
}

function denyForbidden(state: bigint, forbidden: bigint): bigint {
  // forbidden_present = state & forbidden
  return state & forbidden;
}

function boolMask(flag: boolean): bigint {
  return flag ? 0xffffffffffffffffn : 0n;
}

/** Pure-TS motion_tick. */
export function motionTick(req: MotionRequest): MotionResponse {
  const { state, fields, instructionId } = req;

  const perLane: Record<FieldLaneName, bigint> = {
    prereq: boolMask(denyRequired(state, fields.prereq_required) !== 0n),
    capability: boolMask(
      denyRequired(state, fields.capability_required) !== 0n
    ),
    causality: boolMask(denyRequired(state, fields.causality_required) !== 0n),
    conformance: boolMask(
      denyRequired(state, fields.conformance_required) !== 0n
    ),
    law: boolMask(denyForbidden(state, fields.law_forbidden) !== 0n),
    scenario: boolMask(denyForbidden(state, fields.scenario_forbidden) !== 0n),
    risk: boolMask(denyForbidden(state, fields.risk_forbidden) !== 0n),
    attention: boolMask(
      denyForbidden(state, fields.attention_forbidden) !== 0n
    ),
  };

  let denyTotal = 0n;
  for (const lane of LANES) {
    denyTotal |= perLane[lane];
  }

  // Branchless select: next = (candidate & admitted_mask) | (old & !admitted_mask)
  // For the demo, candidate is simply state unchanged.
  const admitted = denyTotal === 0n;
  const admittedMask = boolMask(admitted);
  const candidate = state;
  const nextMarking = (candidate & admittedMask) | (state & ~admittedMask);

  // Simple fragment: (denyTotal ^ instructionId ^ nextMarking)
  const fragment =
    (denyTotal ^ instructionId ^ nextMarking) & 0xffffffffffffffffn;

  return {
    nextMarking,
    denyTotal,
    fragment,
    status: 0,
    perLane,
  };
}

/** Canonical eight-field masks matching unibit-e2e's canonical_packed_field. */
export const CANONICAL_FIELDS: PackedEightField = {
  prereq_required: 0b0001n,
  capability_required: 0b0010n,
  causality_required: 0b0100n,
  conformance_required: 0b1000n,
  law_forbidden: 0b0001_0000n,
  scenario_forbidden: 0b0010_0000n,
  risk_forbidden: 0b0100_0000n,
  attention_forbidden: 0b1000_0000n,
};
