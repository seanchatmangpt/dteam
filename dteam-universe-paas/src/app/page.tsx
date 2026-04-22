"use client";

import { useEffect } from "react";
import { BlackHoleEHT } from "@/components/3d/BlackHoleEHT";
import { ReceiptPanel } from "@/components/ui/ReceiptPanel";
import { ControlPanel } from "@/components/ui/ControlPanel";
import { useProjectionStore } from "@/store/projectionStore";

export default function UniverseDashboard() {
  const { projection, connectToUniverse } = useProjectionStore();

  useEffect(() => {
    connectToUniverse();
  }, [connectToUniverse]);

  if (!projection) return <div className="bg-black h-screen w-screen" />;

  return (
    <main className="relative h-screen w-screen bg-[#02030a] overflow-hidden font-mono text-white/80 select-none">
      {/* 3D Canvas Layer */}
      <div className="absolute inset-0 z-0">
        <BlackHoleEHT projection={projection} />
      </div>

      {/* HUD Layer */}
      <div className="absolute inset-0 z-10 pointer-events-none flex flex-col justify-between p-6">
        
        {/* Top Header */}
        <header className="flex justify-between items-start">
          <div>
            <h1 className="text-xl font-bold tracking-widest text-white">DTEAM // UNIVERSE_OS</h1>
            <p className="text-xs text-orange-500/80 mt-1 uppercase tracking-widest">
              Black-box process intelligence. White-box proof.
            </p>
          </div>
          <div className="flex items-center gap-3">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
            </span>
            <span className="text-xs tracking-wider">L1_RESIDENT_SYNC</span>
          </div>
        </header>

        {/* Bottom Panels */}
        <div className="flex justify-between items-end pointer-events-auto">
          <ControlPanel />
          <ReceiptPanel receipt={projection.receipt} />
        </div>
      </div>
    </main>
  );
}