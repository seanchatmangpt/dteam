#!/bin/bash
set -e

# ccog Benchmarking DX Agent - Summary Report Generator
echo "Starting ccog benchmark suite..."

# Ensure we are in the crate root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run from the crate root."
    exit 1
fi

mkdir -p target/bench-reports

BENCHMARKS=(
    "ccog_hot_path_bench"
    "pack_lifestyle_bench"
    "pack_edge_bench"
    "pack_enterprise_bench"
    "pack_dev_bench"
    "kernel_bench"
    "dx_bench"
)

REPORT_FILE="target/bench-reports/summary.md"
echo "# ccog Benchmark Summary Report" > "$REPORT_FILE"
echo "Generated on: $(date)" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

for bench in "${BENCHMARKS[@]}"; do
    echo "--> Running $bench..."
    # Capture only the relevant timing lines
    cargo bench --bench "$bench" > "target/bench-reports/${bench}.log" 2>&1
    
    echo "## $bench" >> "$REPORT_FILE"
    echo " \`\`\` " >> "$REPORT_FILE"
    grep -E "time:   \[" "target/bench-reports/${bench}.log" >> "$REPORT_FILE" || echo "No timing data found." >> "$REPORT_FILE"
    echo " \`\`\` " >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
done

echo "Done! Summary report generated at $REPORT_FILE"
cat "$REPORT_FILE"
