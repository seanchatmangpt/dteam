export type VerificationStatus = "Verified" | "Pending" | "Rejected";

export interface DTeamReceipt {
  id: string;
  challengeId: string;
  inputCommitment: string;
  outputCommitment: string;
  replayCommitment: string;
  minimalityScore: number | null;
  status: VerificationStatus;
  timestamp: string;
}

export type BlackHolePreset = "M87*" | "Sagittarius A*";

export interface BlackHoleProjection {
  object: BlackHolePreset;
  projectionKind: "eht-style-boundary-object";
  receipt: DTeamReceipt;
  ring: {
    radius: number;
    thickness: number;
    asymmetry: number;
    turbulence: number;
  };
  overlays: {
    polarization: boolean;
    jetAxis: boolean;
    referenceGrid: boolean;
  };
}