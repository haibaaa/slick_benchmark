#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "matplotlib",
#   "pandas",
# ]
# ///
"""Track performance as load factor grows from 0 to 1.0 with fixed capacity.

Replicates the paper's methodology: fixed capacity (default 2M), gradually
increase elements inserted to track behavior as LF goes 0 -> 1.0.
"""

import argparse
import subprocess
import sys
from pathlib import Path

import matplotlib
matplotlib.use('Agg')  # Non-interactive backend for headless environments
import matplotlib.pyplot as plt
import pandas as pd

# Project root is parent of scripts/ directory
PROJECT_ROOT = Path(__file__).resolve().parent.parent
RESULTS_DIR = PROJECT_ROOT / "results"
RESULTS_CSV = RESULTS_DIR / "results_lf_growth.csv"
PLOTS_DIR = PROJECT_ROOT / "plots"


def print_command(command: list[str]) -> None:
    """Print a command exactly as it will be executed."""
    print(f"[lf_growth] $ {' '.join(command)}")


def run_benchmark(dataset: str, workload: str, size: int, capacity: int, output_csv: str) -> None:
    """Execute one benchmark run with fixed capacity, varying size to control LF."""
    cmd = [
        "cargo", "run", "--release", "--",
        "--dataset", dataset,
        "--workload", workload,
        "--size", str(size),
        "--initial-capacity", str(capacity),
        "--output", output_csv,
    ]
    print_command(cmd)
    result = subprocess.run(cmd, text=True, capture_output=True, cwd=PROJECT_ROOT)
    
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    
    if result.returncode != 0:
        raise subprocess.CalledProcessError(
            result.returncode, cmd, output=result.stdout, stderr=result.stderr
        )


def generate_plots(csv_path: Path, dataset: str, workload: str) -> None:
    """Generate performance plots with continuous load factor (0->1.0) on x-axis."""
    print(f"[lf_growth] Loading results from {csv_path}")
    df = pd.read_csv(csv_path)
    
    # Calculate actual load factor = elements / capacity
    df["load_factor"] = df["elements"] / df["capacity"]
    
    # Remove duplicates: keep last run for each (dataset, workload, table, elements)
    df = df.drop_duplicates(
        subset=["dataset", "workload", "table", "elements"],
        keep="last"
    )
    
    # Filter to target dataset and workload
    subset = df[(df["dataset"] == dataset) & (df["workload"] == workload)]
    if subset.empty:
        print(f"[lf_growth] No data for dataset={dataset}, workload={workload}")
        return
    
    PLOTS_DIR.mkdir(exist_ok=True)
    tables = sorted(subset["table"].unique())
    
    # Get load factor range for x-axis
    lf_min = subset["load_factor"].min()
    lf_max = subset["load_factor"].max()
    print(f"[lf_growth] Load factor range: {lf_min:.3f} -> {lf_max:.3f}")
    
    # Create figure with 3 subplots
    fig, axes = plt.subplots(1, 3, figsize=(24, 6), constrained_layout=True)
    
    # Plot 1: Insert performance
    ax = axes[0]
    for table in tables:
        table_data = subset[subset["table"] == table].sort_values("load_factor")
        ax.plot(
            table_data["load_factor"],
            table_data["insert_ns_per_op"],
            marker='o',
            label=table,
            linewidth=2,
        )
    ax.set_title(f"{dataset}/{workload}: Insert ns/op")
    ax.set_xlabel("Load Factor (0 -> 1.0)")
    ax.set_ylabel("ns per operation")
    ax.set_xlim(max(0, lf_min * 0.9), min(1.0, lf_max * 1.1))
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 2: Find performance
    ax = axes[1]
    for table in tables:
        table_data = subset[subset["table"] == table].sort_values("load_factor")
        ax.plot(
            table_data["load_factor"],
            table_data["find_ns_per_op"],
            marker='o',
            label=table,
            linewidth=2,
        )
    ax.set_title(f"{dataset}/{workload}: Find ns/op")
    ax.set_xlabel("Load Factor (0 -> 1.0)")
    ax.set_ylabel("ns per operation")
    ax.set_xlim(max(0, lf_min * 0.9), min(1.0, lf_max * 1.1))
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 3: Space efficiency
    ax = axes[2]
    for table in tables:
        table_data = subset[subset["table"] == table].sort_values("load_factor")
        ax.plot(
            table_data["load_factor"],
            table_data["bytes_per_element"],
            marker='o',
            label=table,
            linewidth=2,
        )
    ax.set_title(f"{dataset}/{workload}: Space Efficiency")
    ax.set_xlabel("Load Factor (0 -> 1.0)")
    ax.set_ylabel("bytes per element")
    ax.set_xlim(max(0, lf_min * 0.9), min(1.0, lf_max * 1.1))
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Save plot
    output_path = PLOTS_DIR / f"lf_growth_{dataset}_{workload}.png"
    fig.savefig(output_path, dpi=150)
    plt.close(fig)
    print(f"[lf_growth] Saved plot: {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Track performance as load factor grows 0->1.0 (paper replication)"
    )
    parser.add_argument(
        "--dataset",
        choices=["uniform", "sequential", "zipf", "norvig", "wikipedia"],
        default="uniform",
        help="Dataset to benchmark (default: uniform)",
    )
    parser.add_argument(
        "--workload",
        choices=["bulk", "mixed", "read_heavy"],
        default="bulk",
        help="Workload to run (default: bulk)",
    )
    parser.add_argument(
        "--capacity",
        type=int,
        default=2_000_000,
        help="Fixed capacity (paper uses 2M, default: 2M)",
    )
    parser.add_argument(
        "--steps",
        type=int,
        default=20,
        help="Number of LF steps from 0 to 1.0 (default: 20 for continuous graph)",
    )
    parser.add_argument(
        "--output",
        default="results_lf_growth.csv",
        help="Output CSV file (default: results_lf_growth.csv)",
    )
    parser.add_argument(
        "--fresh",
        action="store_true",
        help="Delete existing results CSV before running",
    )
    parser.add_argument(
        "--plot-only",
        action="store_true",
        help="Skip benchmarking, only plot existing results",
    )
    args = parser.parse_args()
    
    # Resolve output path
    output_path = Path(args.output)
    if not output_path.is_absolute():
        output_path = PROJECT_ROOT / args.output
    
    # Delete old CSV if fresh flag is set
    if args.fresh and RESULTS_CSV.exists():
        RESULTS_CSV.unlink()
        print(f"[lf_growth] Deleted existing {RESULTS_CSV} for fresh run")
    
    # Ensure results directory exists
    RESULTS_DIR.mkdir(exist_ok=True)
    
    # Run benchmarks if not plot-only
    if not args.plot_only:
        print(f"[lf_growth] Fixed capacity: {args.capacity}")
        print(f"[lf_growth] Steps: {args.steps} (LF 0 -> 1.0)")
        
        for i in range(1, args.steps + 1):
            # Calculate size to get LF = i/steps
            # LF = elements / capacity => elements = LF * capacity
            target_lf = i / args.steps
            size = int(target_lf * args.capacity)
            print(f"\n[lf_growth] Step {i}/{args.steps}: size={size}, LF={target_lf:.2f}")
            run_benchmark(args.dataset, args.workload, size, args.capacity, str(output_path))
    
    # Generate plots
    if not output_path.exists():
        raise FileNotFoundError(f"No results file found: {output_path}")
    
    generate_plots(output_path, args.dataset, args.workload)


if __name__ == "__main__":
    main()
