"use client";

import { Canvas } from "@react-three/fiber";
import { OrbitControls, Stars } from "@react-three/drei";
import { BlackHoleProjection } from "@/types/universe";
import { AccretionRing } from "./AccretionRing";
import { PhotonShimmer } from "./PhotonShimmer";
import { PolarizationField } from "./PolarizationField";
import { RelativisticJet } from "./RelativisticJet";
import { ReferenceGrid } from "./ReferenceGrid";

export function BlackHoleEHT({ projection }: { projection: BlackHoleProjection }) {
  return (
    <Canvas camera={{ position: [0, 0.8, 7], fov: 45 }} gl={{ antialias: true, preserveDrawingBuffer: true }}>
      <color attach="background" args={["#02030a"]} />
      <ambientLight intensity={0.25} />
      <pointLight position={[4, 4, 5]} intensity={2.0} />
      
      <Stars radius={80} depth={40} count={1800} factor={3} saturation={0} fade speed={0.5} />

      {/* The Shadow Core */}
      <mesh>
        <sphereGeometry args={[1.12, 96, 96]} />
        <meshBasicMaterial color="#000000" />
      </mesh>

      <AccretionRing params={projection.ring} />
      <PhotonShimmer params={projection.ring} />
      
      {projection.overlays.polarization && <PolarizationField params={projection.ring} />}
      {projection.overlays.jetAxis && <RelativisticJet />}
      {projection.overlays.referenceGrid && <ReferenceGrid />}

      <OrbitControls enablePan={false} minDistance={3.5} maxDistance={12} autoRotate autoRotateSpeed={0.5} />
    </Canvas>
  );
}