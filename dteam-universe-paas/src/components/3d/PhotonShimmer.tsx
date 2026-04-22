"use client";

import { useMemo, useRef } from "react";
import * as THREE from "three";
import { useFrame } from "@react-three/fiber";

export function PhotonShimmer({ params }: { params: { radius: number; thickness: number } }) {
  const group = useRef<THREE.Group>(null);

  const particles = useMemo(() => {
    const points: THREE.Vector3[] = [];

    for (let i = 0; i < 900; i++) {
      const t = Math.random() * Math.PI * 2;
      const jitter = (Math.random() - 0.5) * params.thickness;
      const r = params.radius + jitter;

      points.push(
        new THREE.Vector3(
          Math.cos(t) * r * 1.05, // Slight elliptical orbit
          Math.sin(t) * r * 0.88,
          (Math.random() - 0.5) * 0.08 // Tight Z-axis bounds
        )
      );
    }

    return points;
  }, [params]);

  useFrame(({ clock }) => {
    if (!group.current) return;
    // Slow, deterministic rotation
    group.current.rotation.z = clock.elapsedTime * 0.035;
  });

  return (
    <group ref={group}>
      {particles.map((p, i) => (
        <mesh key={i} position={p}>
          <sphereGeometry args={[0.006 + Math.random() * 0.012, 6, 6]} />
          <meshBasicMaterial color="#ffb14a" transparent opacity={0.28} />
        </mesh>
      ))}
    </group>
  );
}