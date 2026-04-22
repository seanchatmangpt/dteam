import { create } from 'zustand';
import { BlackHoleProjection } from '../types/universe';

interface ProjectionState {
  projection: BlackHoleProjection | null;
  isConnected: boolean;
  connectToUniverse: () => void;
  setProjection: (payload: BlackHoleProjection) => void;
}

const M87_INITIAL: BlackHoleProjection = {
  object: "M87*",
  projectionKind: "eht-style-boundary-object",
  receipt: {
    id: "dteam:eht:m87:proj:0001",
    challengeId: "chal:1001",
    inputCommitment: "0xbf58476d1ce4e5b9",
    outputCommitment: "0x94d049bb133111eb",
    replayCommitment: "0xD37AD37AD37AD37A",
    minimalityScore: 1.0,
    status: "Verified",
    timestamp: new Date().toISOString(),
  },
  ring: { radius: 2.25, thickness: 0.42, asymmetry: 0.82, turbulence: 0.18 },
  overlays: { polarization: true, jetAxis: true, referenceGrid: true },
};

export const useProjectionStore = create<ProjectionState>((set) => ({
  projection: M87_INITIAL,
  isConnected: false,
  connectToUniverse: () => {
    // In reality: Connect to AtomVM boundary SSE
    set({ isConnected: true });
  },
  setProjection: (payload) => set({ projection: payload }),
}));