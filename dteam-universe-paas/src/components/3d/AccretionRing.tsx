"use client";

import { useMemo, useRef } from "react";
import * as THREE from "three";
import { useFrame } from "@react-three/fiber";

export function AccretionRing({ params }: { params: { radius: number; thickness: number; asymmetry: number; turbulence: number } }) {
  const mesh = useRef<THREE.Mesh>(null);

  const material = useMemo(() => {
    return new THREE.ShaderMaterial({
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
      side: THREE.DoubleSide,
      uniforms: {
        uTime: { value: 0 },
        uRingRadius: { value: params.radius },
        uShadowRadius: { value: 1.12 }, // Fixed shadow for M87 baseline
        uThickness: { value: params.thickness },
        uAsymmetry: { value: params.asymmetry },
        uTurbulence: { value: params.turbulence },
      },
      vertexShader: `
        varying vec2 vUv;
        void main() {
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        precision highp float;
        uniform float uTime;
        uniform float uRingRadius;
        uniform float uThickness;
        uniform float uAsymmetry;
        uniform float uTurbulence;
        varying vec2 vUv;

        float hash(vec2 p) {
          p = fract(p * vec2(123.34, 456.21));
          p += dot(p, p + 45.32);
          return fract(p.x * p.y);
        }

        float noise(vec2 p) {
          vec2 i = floor(p);
          vec2 f = fract(p);
          float a = hash(i);
          float b = hash(i + vec2(1.0, 0.0));
          float c = hash(i + vec2(0.0, 1.0));
          float d = hash(i + vec2(1.0, 1.0));
          vec2 u = f * f * (3.0 - 2.0 * f);
          return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
        }

        void main() {
          vec2 p = vUv * 2.0 - 1.0;
          p.x *= 1.18; // Slight elliptical stretch

          float r = length(p);
          float a = atan(p.y, p.x);

          float swirl = noise(vec2(a * 3.0 + uTime * 0.15, r * 6.0));
          float ringCenter = (uRingRadius * 0.25) + 0.035 * sin(a * 3.0 + uTime * 0.5);
          float ring = exp(-pow((r - ringCenter) / uThickness, 2.0));

          float shadow = smoothstep(0.28, 0.38, r);
          float beaming = 0.55 + uAsymmetry * max(0.0, cos(a + 2.45)); // Doppler boost
          float turbulence = 0.78 + uTurbulence * swirl;

          float intensity = ring * shadow * beaming * turbulence;

          // Thermal ramp
          vec3 darkRed = vec3(0.25, 0.035, 0.005);
          vec3 orange = vec3(1.0, 0.33, 0.035);
          vec3 yellow = vec3(1.0, 0.78, 0.20);
          vec3 whiteHot = vec3(1.0, 0.92, 0.68);

          vec3 color = mix(darkRed, orange, intensity);
          color = mix(color, yellow, smoothstep(0.45, 0.85, intensity));
          color = mix(color, whiteHot, smoothstep(0.82, 1.15, intensity));

          float alpha = smoothstep(0.03, 0.12, intensity) * 0.95;
          gl_FragColor = vec4(color * intensity * 1.8, alpha);
        }
      `,
    });
  }, [params]);

  useFrame(({ clock }) => {
    material.uniforms.uTime.value = clock.elapsedTime;
    if (mesh.current) {
      mesh.current.rotation.z = Math.sin(clock.elapsedTime * 0.05) * 0.04;
    }
  });

  return (
    <mesh ref={mesh} rotation={[0.2, 0.0, 0.0]}>
      <planeGeometry args={[5.6, 5.6, 1, 1]} />
      <primitive object={material} attach="material" />
    </mesh>
  );
}