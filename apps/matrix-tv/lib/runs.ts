/**
 * The 44 runs from the Sprawl trilogy, mapped to unibit arenas
 * (see docs/opus/60). Each run is an input state, an expected
 * outcome, and camera choreography.
 */

import { CANONICAL_FIELDS, MotionRequest } from './unibit';

export interface CameraKeyframe {
  pos: [number, number, number];
  look: [number, number, number];
  t: number;
}

export interface Annotation {
  t: number;
  text: string;
}

export interface Run {
  id: string;
  title: string;
  source: 'Neuromancer' | 'Count Zero' | 'Mona Lisa Overdrive' | 'unibit';
  arena: string;
  request: MotionRequest;
  expected: {
    admitted: boolean;
    description: string;
  };
  camera: CameraKeyframe[];
  annotations: Annotation[];
}

// =============================================================================
// Neuromancer
// =============================================================================

const NEUROMANCER: Run[] = [
  {
    id: 'N1',
    title: "Case's first jack-in at the Chiba clinic",
    source: 'Neuromancer',
    arena: 'arena_15_l1_region',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0101n,
    },
    expected: { admitted: true, description: 'HotRegion materialises; L1 position receipt seals' },
    camera: [
      { pos: [0, 0, 10], look: [0, 0, 0], t: 0 },
      { pos: [3, 2, 4], look: [0, 0, 0], t: 3 },
    ],
    annotations: [
      { t: 0.5, text: 'Pin<Box<L1Region>> allocated' },
      { t: 2.0, text: 'mlock succeeds; position validated' },
    ],
  },
  {
    id: 'N2',
    title: 'Dixie Flatline construct replay',
    source: 'Neuromancer',
    arena: 'arena_03_snapshot_nesting',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0102n,
    },
    expected: { admitted: true, description: 'Nested Snapshot verifies deep' },
    camera: [
      { pos: [0, 0, 8], look: [0, 0, 0], t: 0 },
      { pos: [0, 5, 2], look: [0, 0, 0], t: 5 },
    ],
    annotations: [
      { t: 1.0, text: 'Snapshot inner loads' },
      { t: 3.0, text: 'BLAKE3 seals re-derive' },
    ],
  },
  {
    id: 'N3',
    title: 'First ICE encounter (Sense/Net)',
    source: 'Neuromancer',
    arena: 'arena_17_hot_kernels',
    request: {
      state: 0b0001_1111n, // law-forbidden bit 4 is set
      fields: CANONICAL_FIELDS,
      instructionId: 0x0103n,
    },
    expected: { admitted: false, description: 'Law lane fires a red flare' },
    camera: [
      { pos: [2, 0, 5], look: [0, 0, 0], t: 0 },
      { pos: [-2, 1, 3], look: [0, 0, 0], t: 4 },
    ],
    annotations: [
      { t: 1.5, text: 'Law mask matches — denial' },
      { t: 2.5, text: 'Fragment emitted; Watchdog ticked' },
    ],
  },
  {
    id: 'N6',
    title: "Ratz's bar, the scorched trodes",
    source: 'Neuromancer',
    arena: 'arena_04_watchdog_liveness',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0106n,
    },
    expected: {
      admitted: true,
      description: 'Watchdog ring dims; recovers via tick',
    },
    camera: [{ pos: [0, 0, 6], look: [0, 0, 0], t: 0 }],
    annotations: [{ t: 1.0, text: 'Watchdog countdown visible' }],
  },
  {
    id: 'N7',
    title: 'Turing police raid on the Villa',
    source: 'Neuromancer',
    arena: 'arena_07_chain_replay',
    request: {
      state: 0b1111_0000n, // all forbidden bits set, no required
      fields: CANONICAL_FIELDS,
      instructionId: 0x0107n,
    },
    expected: {
      admitted: false,
      description: 'Every lane denies; quarantine halo',
    },
    camera: [{ pos: [0, 0, 7], look: [0, 0, 0], t: 0 }],
    annotations: [{ t: 1.0, text: 'All 8 lanes fire simultaneously' }],
  },
];

// =============================================================================
// Count Zero
// =============================================================================

const COUNT_ZERO: Run[] = [
  {
    id: 'CZ1',
    title: "Bobby Newmark's first run, near-flatline",
    source: 'Count Zero',
    arena: 'arena_04_watchdog_liveness',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0201n,
    },
    expected: { admitted: true, description: 'Watchdog almost zeroes; tick saves the run' },
    camera: [{ pos: [0, 0, 6], look: [0, 0, 0], t: 0 }],
    annotations: [{ t: 1.5, text: 'Counter: 3... 2... 1... tick.' }],
  },
  {
    id: 'CZ3',
    title: 'Legba rides Bobby',
    source: 'Count Zero',
    arena: 'arena_31_commandeering_override',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0203n,
    },
    expected: {
      admitted: true,
      description: 'Prereq lane commandeers; color saturates',
    },
    camera: [{ pos: [0, 0, 8], look: [0, 0, 0], t: 0 }],
    annotations: [{ t: 1.0, text: 'Prereq lane mode: Commandeering' }],
  },
  {
    id: 'CZ4',
    title: "The Boxmaker's assemblages",
    source: 'Count Zero',
    arena: 'arena_35_orphan_assembler',
    request: {
      state: 0b1111_0000n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0204n,
    },
    expected: {
      admitted: false,
      description: 'Fragments scatter; Boxmaker folds them into a Snapshot',
    },
    camera: [
      { pos: [0, 0, 10], look: [0, 0, 0], t: 0 },
      { pos: [4, 0, 3], look: [0, 0, 0], t: 4 },
    ],
    annotations: [
      { t: 1.0, text: 'Fragments dropped into the orphan pool' },
      { t: 3.0, text: 'Snapshot seal computes' },
    ],
  },
];

// =============================================================================
// Mona Lisa Overdrive
// =============================================================================

const MONA_LISA_OVERDRIVE: Run[] = [
  {
    id: 'MLO2',
    title: 'The Aleph — Straylight recreated',
    source: 'Mona Lisa Overdrive',
    arena: 'arena_03_snapshot_nesting',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0302n,
    },
    expected: {
      admitted: true,
      description: 'Deep-nested Snapshot to MAX_DEPTH = 16',
    },
    camera: [
      { pos: [0, 0, 10], look: [0, 0, 0], t: 0 },
      { pos: [0, 0, 2], look: [0, 0, 0], t: 8 },
    ],
    annotations: [
      { t: 2.0, text: 'Camera descends Aleph.inner' },
      { t: 6.0, text: 'Innermost Snapshot reached' },
    ],
  },
  {
    id: 'MLO6',
    title: "The Count's return",
    source: 'Mona Lisa Overdrive',
    arena: 'arena_08_publish_readiness',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x0306n,
    },
    expected: {
      admitted: true,
      description: 'Two runs side-by-side; receipts match bit-for-bit',
    },
    camera: [{ pos: [0, 0, 8], look: [0, 0, 0], t: 0 }],
    annotations: [{ t: 1.0, text: 'Deterministic replay — identical seals' }],
  },
  {
    id: 'MLO10',
    title: "The Jammer's last run",
    source: 'Mona Lisa Overdrive',
    arena: 'arena_33_publish_checklist',
    request: {
      state: 0b0000_1111n,
      fields: CANONICAL_FIELDS,
      instructionId: 0x030an,
    },
    expected: {
      admitted: true,
      description: 'Every indicator greens; five-word manifesto audit passes',
    },
    camera: [{ pos: [0, 0, 8], look: [0, 0, 0], t: 0 }],
    annotations: [
      { t: 0.5, text: 'pinned ✓' },
      { t: 1.0, text: 'branchless ✓' },
      { t: 1.5, text: 'typed ✓' },
      { t: 2.0, text: 'receipted ✓' },
      { t: 2.5, text: 'narrow ✓' },
    ],
  },
];

export const ALL_RUNS: Run[] = [
  ...NEUROMANCER,
  ...COUNT_ZERO,
  ...MONA_LISA_OVERDRIVE,
];

export function runBySlug(slug: string): Run | undefined {
  return ALL_RUNS.find((r) => r.id.toLowerCase() === slug.toLowerCase());
}
