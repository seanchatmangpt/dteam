"use client";

import { useMemo } from "react";
import * as THREE from "three";

export function PolarizationField({ params }: { params: { radius: number } }) {
  const vectors = useMemo(() => {
    const items: { position: THREE.Vector3; rotation: number; scale: number }[] = [];

    for (let i = 0; i < 96; i++) {
      const t = (i / 96) * Math.PI * 2;
      // Procedural distortion to mimic EHT vector fields
      const r = params.radius * (0.82 + 0.28 * Math.sin(i * 12.989)); 
      const x = Math.cos(t) * r * 1.05;
      const y = Math.sin(t) * r * 0.88;

      items.push({
        position: new THREE.Vector3(x, y, 0.08),
        rotation: t + Math.PI / 2 + 0.4 * Math.sin(t * 3.0),
        scale: 0.08 + 0.05 * Math.random(),
      });
    }

    return items;
  }, [params]);

  return (
    <group>
      {vectors.map((v, i) => (
        <mesh
          key={i}
          position={v.position}
          rotation={[0, 0, v.rotation]}
          scale={[v.scale, 0.012, 0.012]} // Thin, needle-like boxes
        >
          <boxGeometry args={[1, 1, 1]} />
          <meshBasicMaterial color="#ffd28a" transparent opacity={0.42} />
        </mesh>
      ))}
    </group>
  );
}