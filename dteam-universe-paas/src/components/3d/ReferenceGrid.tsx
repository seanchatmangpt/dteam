"use client";

export function ReferenceGrid() {
  return (
    <group rotation={[Math.PI / 2, 0, 0]} position={[0, 0, -0.08]}>
      {/* Outer limits frame and fine internal division */}
      <gridHelper args={[8, 32, "#223355", "#101827"]} />
    </group>
  );
}