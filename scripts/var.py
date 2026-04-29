#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "matplotlib",
#   "pandas",
# ]
# ///
"""Variable workload benchmark: 10x increasing dataset size, plot performance."""

import argparse
import subprocess
import sys
from pathlib import Path

import matplotlib
matplotlib.use('Agg')  # Non-interactive backend for headless environments
import matplotlib.pyplot as plt
import pandas as pd

# Configuration
DATASETS = ["uniform", "sequential", "zipf", "norvig", "wikipedia"]
WORKLOADS = ["bulk", "mixed", "read_heavy"]

# Project root is parent of scripts/ directory
PROJECT_ROOT = Path(__file__).resolve().parent.parent
RESULTS_DIR = PROJECT_ROOT / "results"
PLOTS_DIR = PROJECT_ROOT / "plots"
DEFAULT_OUTPUT = str(RESULTS_DIR / "results_var.csv")


def print_command(command: list[str]) -> None:
    """Print a command exactly as it will be executed."""
    print(f"[var.py] $ {' '.join(command)}")


def run_benchmark(dataset: str, workload: str, size: int, output_csv: str, initial_capacity: int = None) -> None:
    """Execute one benchmark run with given parameters."""
    cmd = [
        "cargo", "run", "--release", "--",
        "--dataset", dataset,
        "--workload", workload,
        "--size", str(size),
        "--output", output_csv,
    ]
    if initial_capacity:
        cmd.extend(["--initial-capacity", str(initial_capacity)])
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
    """Generate performance plots with total ops (10x steps) on x-axis."""
    print(f"[var.py] Loading results from {csv_path}")
    df = pd.read_csv(csv_path)
    
    # Calculate total operations
    df["total_ops"] = df["insert_count"] + df["find_count"]
    
    # Remove duplicates: keep last run for each (dataset, workload, table, total_ops)
    df = df.drop_duplicates(
        subset=["dataset", "workload", "table", "total_ops"], 
        keep="last"
    )
    
    # Filter to target dataset and workload
    subset = df[(df["dataset"] == dataset) & (df["workload"] == workload)]
    if subset.empty:
        print(f"[var.py] No data for dataset={dataset}, workload={workload}")
        return
    
    PLOTS_DIR.mkdir(exist_ok=True)
    tables = sorted(subset["table"].unique())
    
    # Get unique total_ops values for x-axis ticks
    x_values = sorted(subset["total_ops"].unique())
    print(f"[var.py] X-axis values: {x_values}")
    
    # Format tick labels (k, M, B format)
    def format_tick(x):
        if x < 1e3:
            return f"{int(x)}"
        elif x < 1e6:
            return f"{int(x/1e3)}k"
        elif x < 1e9:
            return f"{int(x/1e6)}M"
        else:
            return f"{int(x/1e9)}B"
    
    # Create figure with 3 subplots
    fig, axes = plt.subplots(1, 3, figsize=(24, 6), constrained_layout=True)
    
    # Plot 1: Insert performance
    ax = axes[0]
    for table in tables:
        table_data = subset[subset["table"] == table].sort_values("total_ops")
        ax.plot(
            table_data["total_ops"],
            table_data["insert_ns_per_op"],
            marker='o',
            label=table,
            linewidth=2,
        )
    ax.set_xscale('log', base=10)
    ax.set_xticks(x_values)
    ax.set_xticklabels([format_tick(x) for x in x_values], rotation=45)
    ax.set_xlim(min(x_values) * 0.9, max(x_values) * 1.1)
    ax.set_title(f"{dataset}/{workload}: Insert ns/op")
    ax.set_xlabel("Total Operations (10x steps)")
    ax.set_ylabel("ns per operation")
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 2: Find performance
    ax = axes[1]
    for table in tables:
        table_data = subset[subset["table"] == table].sort_values("total_ops")
        ax.plot(
            table_data["total_ops"],
            table_data["find_ns_per_op"],
            marker='o',
            label=table,
            linewidth=2,
        )
    ax.set_xscale('log', base=10)
    ax.set_xticks(x_values)
    ax.set_xticklabels([format_tick(x) for x in x_values], rotation=45)
    ax.set_xlim(min(x_values) * 0.9, max(x_values) * 1.1)
    ax.set_title(f"{dataset}/{workload}: Find ns/op")
    ax.set_xlabel("Total Operations (10x steps)")
    ax.set_ylabel("ns per operation")
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 3: Space efficiency
    ax = axes[2]
    for table in tables:
        table_data = subset[subset["table"] == table].sort_values("total_ops")
        ax.plot(
            table_data["total_ops"],
            table_data["bytes_per_element"],
            marker='o',
            label=table,
            linewidth=2,
        )
    ax.set_xscale('log', base=10)
    ax.set_xticks(x_values)
    ax.set_xticklabels([format_tick(x) for x in x_values], rotation=45)
    ax.set_xlim(min(x_values) * 0.9, max(x_values) * 1.1)
    ax.set_title(f"{dataset}/{workload}: Space Efficiency")
    ax.set_xlabel("Total Operations (10x steps)")
    ax.set_ylabel("bytes per element")
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Save plot
    output_path = PLOTS_DIR / f"var_{dataset}_{workload}.png"
    fig.savefig(output_path, dpi=150)
    plt.close(fig)
    print(f"[var.py] Saved plot: {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Variable workload benchmark with 10x increasing size"
    )
    parser.add_argument(
        "--dataset",
        choices=DATASETS,
        default="uniform",
        help="Dataset to benchmark (default: uniform)",
    )
    parser.add_argument(
        "--workload",
        choices=WORKLOADS,
        default="bulk",
        help="Workload to run (default: bulk)",
    )
    parser.add_argument(
        "--steps",
        type=int,
        default=4,
        help="Number of 10x workload increments (default: 4)",
    )
    parser.add_argument(
        "--base-size",
        type=int,
        default=10000,
        help="Starting dataset size (default: 10000)",
    )
    parser.add_argument(
        "--output",
        default=DEFAULT_OUTPUT,
        help=f"Output CSV file (default: {DEFAULT_OUTPUT})",
    )
    parser.add_argument(
        "--plot-only",
        action="store_true",
        help="Skip benchmarking, only plot existing results",
    )
    parser.add_argument(
        "--fresh",
        action="store_true",
        help="Delete existing results CSV before running (avoids old data interference)",
    )
    parser.add_argument(
        "--target-lf",
        type=float,
        help="Target load factor (adjusts capacity = size / target_lf)",
    )
    args = parser.parse_args()
    
    # Resolve output path relative to project root if not absolute
    output_path = Path(args.output)
    if not output_path.is_absolute():
        output_path = PROJECT_ROOT / args.output
    
    # Delete old CSV if fresh flag is set
    if args.fresh and output_path.exists():
        output_path.unlink()
        print(f"[var.py] Deleted existing {output_path} for fresh run")
    
    # Run benchmarks if not plot-only
    if not args.plot_only:
        print(f"[var.py] Running {args.steps} steps (10x each) for {args.dataset}/{args.workload}")
        print(f"[var.py] Base size: {args.base_size}, Steps: {args.steps}")
        
        for step in range(args.steps):
            size = args.base_size * (10 ** step)
            # Calculate initial_capacity based on target load factor
            initial_capacity = int(size / args.target_lf) if args.target_lf else None
            print(f"\n[var.py] Step {step + 1}/{args.steps}: size={size}, cap={initial_capacity}")
            run_benchmark(args.dataset, args.workload, size, str(output_path), initial_capacity)
    
    # Generate plots
    if not output_path.exists():
        raise FileNotFoundError(f"No results file found: {output_path}")
    
    generate_plots(output_path, args.dataset, args.workload)


if __name__ == "__main__":
    main()
