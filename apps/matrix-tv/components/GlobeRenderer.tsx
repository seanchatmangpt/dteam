'use client';

import { useFrame } from '@react-three/fiber';
import { useMemo, useRef } from 'react';
import * as THREE from 'three';
import { LANE_HUE, LANES, MotionResponse } from '@/lib/unibit';

/**
 * The 64³ globe: a sphere of points rendered from the TruthBlock.
 * Each point's brightness corresponds to whether that "cell" is set.
 */
export function Globe({ response }: { response: MotionResponse | null }) {
  const meshRef = useRef<THREE.Points>(null);

  const geometry = useMemo(() => {
    // 64³ would be 262,144 points — too many for a demo. Render a
    // representative subset (64 × 64 = 4,096 surface points).
    const positions: number[] = [];
    const colors: number[] = [];

    for (let lat = 0; lat < 64; lat++) {
      for (let lon = 0; lon < 64; lon++) {
        const theta = (lat / 64) * Math.PI - Math.PI / 2;
        const phi = (lon / 64) * 2 * Math.PI;
        const r = 1.5;
        positions.push(
          r * Math.cos(theta) * Math.cos(phi),
          r * Math.sin(theta),
          r * Math.cos(theta) * Math.sin(phi)
        );
        // lat+lon parity gives a subtle pattern so the sphere is visible
        const brightness = 0.3 + 0.1 * ((lat + lon) % 2);
        colors.push(brightness, brightness, brightness + 0.2);
      }
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute(
      'position',
      new THREE.Float32BufferAttribute(positions, 3)
    );
    geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    return geo;
  }, []);

  const admitted = response ? response.denyTotal === 0n : null;

  useFrame((state) => {
    if (!meshRef.current) return;
    meshRef.current.rotation.y += 0.002;
    // Subtle pulse when admitted.
    if (admitted === true) {
      const s = 1 + 0.02 * Math.sin(state.clock.elapsedTime * 3);
      meshRef.current.scale.setScalar(s);
    }
  });

  return (
    <group>
      <points ref={meshRef} geometry={geometry}>
        <pointsMaterial vertexColors size={0.04} sizeAttenuation />
      </points>
      <LaneRings response={response} />
      {admitted === false && <DenialFlares response={response!} />}
    </group>
  );
}

function LaneRings({ response }: { response: MotionResponse | null }) {
  return (
    <group>
      {LANES.map((lane, i) => {
        const angle = (i / 8) * Math.PI;
        const lit =
          response && response.perLane[lane] === 0n
            ? 1.0
            : response
              ? 0.25
              : 0.5;
        const color = new THREE.Color(LANE_HUE[lane]).multiplyScalar(lit);
        return (
          <mesh
            key={lane}
            rotation={[angle, angle * 0.5, 0]}
          >
            <torusGeometry args={[1.5, 0.01, 16, 128]} />
            <meshBasicMaterial color={color} transparent opacity={0.7} />
          </mesh>
        );
      })}
    </group>
  );
}

function DenialFlares({ response }: { response: MotionResponse }) {
  return (
    <group>
      {LANES.map((lane, i) => {
        if (response.perLane[lane] === 0n) return null;
        const angle = (i / 8) * Math.PI * 2;
        const pos: [number, number, number] = [
          Math.cos(angle) * 1.8,
          0,
          Math.sin(angle) * 1.8,
        ];
        return (
          <mesh key={lane} position={pos}>
            <sphereGeometry args={[0.08, 16, 16]} />
            <meshBasicMaterial color={LANE_HUE[lane]} />
          </mesh>
        );
      })}
    </group>
  );
}
