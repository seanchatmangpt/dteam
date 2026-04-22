"use client";

import { useRef } from "react";
import * as THREE from "three";
import { useFrame } from "@react-three/fiber";

export function RelativisticJet() {
  const jet = useRef<THREE.Mesh>(null);

  useFrame(({ clock }) => {
    if (jet.current) {
      // Pulsing opacity tied to deterministic frame clock
      (jet.current.material as THREE.MeshBasicMaterial).opacity = 
        0.12 + 0.04 * Math.sin(clock.elapsedTime * 2.0);
    }
  });

  return (
    <group rotation={[0.35, 0.0, -0.4]}>
      <mesh ref={jet} position={[0, 2.8, -0.35]}>
        <coneGeometry args={[0.28, 5.8, 48, 1, true]} />
        <meshBasicMaterial
          color="#6aa8ff"
          transparent
          opacity={0.14}
          blending={THREE.AdditiveBlending}
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>
    </group>
  );
}