"use client";

import { DTeamReceipt } from "@/types/universe";
import { CheckCircle2, ShieldCheck, Fingerprint, Activity } from "lucide-react";

export function ReceiptPanel({ receipt }: { receipt: DTeamReceipt }) {
  return (
    <div className="bg-[#0a0e17]/80 backdrop-blur-md border border-[#1d283a] p-4 rounded shadow-2xl w-96 flex flex-col gap-4">
      
      <div className="flex items-center justify-between border-b border-[#1d283a] pb-2">
        <div className="flex items-center gap-2 text-emerald-400">
          <ShieldCheck size={16} />
          <span className="font-bold tracking-widest text-sm uppercase">Verified Projection</span>
        </div>
        <span className="text-xs text-white/50">{new Date(receipt.timestamp).toLocaleTimeString()}</span>
      </div>

      <div className="space-y-3 text-xs tracking-wider">
        <div className="flex justify-between items-center">
          <span className="text-white/40">RECEIPT_ID</span>
          <span className="text-white/90 truncate w-48 text-right">{receipt.id}</span>
        </div>
        
        <div className="flex justify-between items-center">
          <span className="text-white/40">INPUT_HASH</span>
          <span className="text-white/90 font-mono">{receipt.inputCommitment}</span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-white/40">REPLAY_CHAIN</span>
          <span className="text-white/90 font-mono">{receipt.replayCommitment}</span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-white/40">MDL_MINIMALITY</span>
          <span className="text-white/90">{receipt.minimalityScore?.toFixed(4)}</span>
        </div>
      </div>

      <div className="bg-[#05080f] border border-[#1d283a] rounded p-3 flex flex-col gap-2 mt-2">
        <div className="flex items-center gap-2">
          <CheckCircle2 size={14} className="text-emerald-500" />
          <span className="text-xs text-emerald-500 uppercase tracking-widest">Lawful Motion Admitted</span>
        </div>
        <div className="flex items-center gap-2">
          <Fingerprint size={14} className="text-emerald-500" />
          <span className="text-xs text-emerald-500 uppercase tracking-widest">Causality Unbroken</span>
        </div>
        <div className="flex items-center gap-2">
          <Activity size={14} className="text-emerald-500" />
          <span className="text-xs text-emerald-500 uppercase tracking-widest">T1_Sparse_Delta Valid</span>
        </div>
      </div>
    </div>
  );
}