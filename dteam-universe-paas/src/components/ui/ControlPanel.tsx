"use client";

import { useProjectionStore } from "@/store/projectionStore";

export function ControlPanel() {
  const { projection } = useProjectionStore();

  return (
    <div className="bg-[#0a0e17]/80 backdrop-blur-md border border-[#1d283a] p-4 rounded w-72 flex flex-col gap-4 shadow-2xl">
      <div className="text-sm font-bold tracking-widest uppercase text-white/80 border-b border-[#1d283a] pb-2">
        Instrument Plane
      </div>

      <div className="space-y-4">
        <div>
          <span className="block text-[10px] text-white/40 uppercase tracking-widest mb-1">Target Object</span>
          <div className="text-sm text-white/90">{projection?.object}</div>
        </div>

        <div>
          <span className="block text-[10px] text-white/40 uppercase tracking-widest mb-2">Semantic Overlays</span>
          <div className="space-y-2">
            <OverlayToggle label="Polarization Vectors" active={projection?.overlays.polarization} />
            <OverlayToggle label="Relativistic Jet" active={projection?.overlays.jetAxis} />
            <OverlayToggle label="Reference Grid" active={projection?.overlays.referenceGrid} />
          </div>
        </div>
      </div>
    </div>
  );
}

function OverlayToggle({ label, active }: { label: string; active?: boolean }) {
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-white/70">{label}</span>
      <span className={active ? "text-orange-400" : "text-white/20"}>{active ? "ON" : "OFF"}</span>
    </div>
  );
}