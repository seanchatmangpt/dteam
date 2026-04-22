# 20 — Browser Black Hole: EHT-Style Projection Demo

## The move

Same projection philosophy as the globe: the rendered object is not the
authority; the receipt is. A browser-native Three.js black hole becomes a
proving ground that the DTEAM projection surface can render extreme
scientific boundary objects, not just enterprise process state.

## Scope

Not a physics simulator. A projection/visualization demo with the familiar
EHT visual structure:

- Dark central shadow (event horizon proxy)
- Asymmetric glowing accretion ring
- Photon-ring shimmer
- Polarization / vector field overlay
- Faint jet axis / reference plane
- Camera orbit controls

## Stack

- `three`, `@react-three/fiber`, `@react-three/drei`
- React component (`react-globe.gl` pattern)
- One custom `ShaderMaterial` for the disc

## Preset table

```typescript
const PRESETS = {
  M87: {
    ringRadius: 2.25, shadowRadius: 1.12, ringThickness: 0.42,
    asymmetry: 0.82, turbulence: 0.18, jetVisible: true,
  },
  SgrA: {
    ringRadius: 1.9, shadowRadius: 0.95, ringThickness: 0.38,
    asymmetry: 0.65, turbulence: 0.28, jetVisible: false,
  },
}
```

## Projection payload

The UI does not invent state. It is fed a verified projection object.

```typescript
export type BlackHoleProjection = {
  object: "M87*" | "Sagittarius A*";
  projectionKind: "eht-style-boundary-object";
  receiptId: string;
  verified: boolean;
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
};
```

## Scene composition

- `BlackHoleCore` — `sphereGeometry` with `meshBasicMaterial` color `#000000`
- `AccretionRing` — `ShaderMaterial` with `uTime`, `uRingRadius`, `uShadowRadius`, `uThickness`, `uAsymmetry`, `uTurbulence` uniforms; additive blending; fragment shader with value noise, ring Gaussian, Doppler-like arc beaming, thermal color ramp (dark red → orange → yellow → white-hot)
- `PhotonShimmer` — 900 particle spheres along the ring circumference with jitter, slow rotation
- `PolarizationField` — 96 oriented bars along the disc with procedural rotation
- `RelativisticJet` (M87 only) — cone geometry with additive blue glow
- `ReferenceGrid` — GridHelper at the disc plane as instrument frame

## Why this is the coup

- EHT itself does not show the object directly; it reconstructs a boundary object from instrument evidence.
- DTEAM does not show the world directly; it projects verified state from admitted evidence.
- **The globe is projection. The black hole ring is projection. The receipt is the authority.**

One-liner:

> If the globe is verified operational geography, the EHT black hole is verified boundary geometry. Same projection philosophy; different universe.

## Accretion disc, not just a black sphere

C4 for the scene mandates the accretion disc as a first-class component —
not decorative geometry around a black sphere. The rendered black hole is a
composed projection object:

```
event horizon / shadow
+ accretion disc
+ photon ring / lensing layer
+ Doppler/asymmetry model
+ turbulence/emissivity shader
+ polarization overlay
+ optional jet/reference frame
```

The design must never reduce to "black sphere in space." The accretion disc is
the main readable object: where the viewer perceives mass, lensing, heat,
asymmetry, motion, and scale.
