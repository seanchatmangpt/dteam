#!/usr/bin/env python3
"""Generate the benchmark TeX tables at an explicit output path."""

from __future__ import annotations

import argparse
from pathlib import Path


def render_performance_tex() -> str:
    agent_data = [
        ("QLearning", "select\\_action", "3.00 ns"),
        ("QLearning", "update", "134.86 ns"),
        ("SARSA", "select\\_action", "5.38 ns"),
        ("SARSA", "update", "136.60 ns"),
        ("DoubleQLearning", "select\\_action", "2.97 ns"),
        ("DoubleQLearning", "update", "248.15 ns"),
        ("ExpectedSARSA", "select\\_action", "3.00 ns"),
        ("ExpectedSARSA", "update", "149.96 ns"),
        ("REINFORCE", "select\\_action", "63.70 ns"),
        ("REINFORCE", "update", "194.03 ns"),
    ]
    algorithm_data = [
        ("XESReader", "read (Domestic)", "142.10 ms"),
        ("PetriNet", "is\\_structural\\_workflow\\_net", "840.00 ns"),
        ("TBR", "Standard Replayer", "6.52 $\\mu$s"),
        ("TBR", "BCINR Optimized Replayer", "4.07 $\\mu$s"),
        ("TBR", "BCINR Pure Bitset Replayer", "975.25 ns"),
    ]

    lines = [
        "\\begin{table}[ht]",
        "\\centering",
        "\\begin{tabular}{llr}",
        "\\toprule",
        "Agent Class & Operation & Latency \\\\",
        "\\midrule",
    ]
    lines.extend(f"{agent} & {operation} & {latency} \\\\" for agent, operation, latency in agent_data)
    lines.extend(
        [
            "\\bottomrule",
            "\\end{tabular}",
            "\\caption{Reinforcement Learning Agent Micro-Benchmarks}",
            "\\label{tab:agent_performance}",
            "\\end{table}",
            "",
            "\\begin{table}[ht]",
            "\\centering",
            "\\begin{tabular}{llr}",
            "\\toprule",
            "Component & Operation & Performance \\\\",
            "\\midrule",
        ]
    )
    lines.extend(f"{component} & {operation} & {performance} \\\\" for component, operation, performance in algorithm_data)
    lines.extend(
        [
            "\\bottomrule",
            "\\end{tabular}",
            "\\caption{Core Process Mining Algorithm Benchmarks}",
            "\\label{tab:algo_performance}",
            "\\end{table}",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path, help="Destination .tex file")
    parser.add_argument("--force", action="store_true", help="Replace an existing output")
    args = parser.parse_args()

    output = args.output.resolve()
    if output.suffix != ".tex":
        parser.error("output must use the .tex extension")
    if output.exists() and not args.force:
        parser.error(f"refusing to overwrite existing file: {output}")

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(render_performance_tex(), encoding="utf-8")
    print(f"generated: {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
