#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "matplotlib",
#   "pandas",
# ]
# ///
"""Benchmark orchestration and plotting pipeline for SlickBench."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd

DATASETS = ["uniform", "sequential", "zipf", "norvig"]
WORKLOADS = ["bulk", "mixed", "read_heavy"]
U64_DATASETS = {"uniform", "sequential", "zipf"}
STRING_DATASETS = {"norvig"}
U64_SIZE = 200_000
STRING_SIZE = 50_000
RESULTS_PATH = Path("results.csv")
PLOTS_DIR = Path("plots")
DATA_DIR = Path("data")
NORVIG_PATH = DATA_DIR / "norvig_words.txt"
WIKI_PATH = DATA_DIR / "wiki_titles.txt"


def print_command(command: list[str]) -> None:
    """Print a command exactly as it will be executed."""
    print(f"[bench.py] $ {' '.join(command)}")


def run_command(
    command: list[str],
    *,
    allow_unsupported_read_heavy: bool = False,
) -> bool:
    """Run a subprocess, optionally treating unsupported workloads as skips."""
    print_command(command)
    result = subprocess.run(command, text=True, capture_output=True)

    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)

    if result.returncode == 0:
        return True

    combined_output = f"{result.stdout}\n{result.stderr}"
    if allow_unsupported_read_heavy and "unsupported workload 'read_heavy'" in combined_output:
        print("[bench.py] Skipping unsupported workload: read_heavy")
        return False

    raise subprocess.CalledProcessError(
        result.returncode,
        command,
        output=result.stdout,
        stderr=result.stderr,
    )


def ensure_datasets() -> None:
    """Download external datasets only when the local files are absent."""
    DATA_DIR.mkdir(exist_ok=True)
    missing = [path for path in (NORVIG_PATH, WIKI_PATH) if not path.exists()]
    if not missing:
        return

    print("[bench.py] Missing datasets detected; downloading prerequisites.")
    run_command(["uv", "run", "scripts/download_data.py"])


def build_release() -> None:
    """Build the Rust benchmark binary once before the benchmark matrix."""
    run_command(["cargo", "build", "--release"])


def dataset_size(dataset: str) -> int:
    """Select the benchmark size appropriate for the dataset key type."""
    if dataset in U64_DATASETS:
        return U64_SIZE
    if dataset in STRING_DATASETS:
        return STRING_SIZE
    raise ValueError(f"unsupported dataset: {dataset}")


def benchmark_command(dataset: str, workload: str, size: int) -> list[str]:
    """Construct the benchmark command in deterministic argument order."""
    return [
        "cargo",
        "run",
        "--release",
        "--",
        "--dataset",
        dataset,
        "--workload",
        workload,
        "--size",
        str(size),
    ]


def run_benchmark(dataset: str, workload: str, size: int) -> bool:
    """Execute one dataset/workload pair, adding `perf stat` when available."""
    command = benchmark_command(dataset, workload, size)
    perf_path = shutil.which("perf")

    if perf_path is None:
        print("[bench.py] perf not found; running benchmark without perf stat.")
        return run_command(command, allow_unsupported_read_heavy=True)

    perf_command = [
        perf_path,
        "stat",
        "-e",
        "cache-misses,branches,branch-misses",
        *command,
    ]
    try:
        return run_command(perf_command, allow_unsupported_read_heavy=True)
    except subprocess.CalledProcessError as exc:
        perf_output = f"{exc.output or ''}\n{exc.stderr or ''}"
        if "unsupported workload 'read_heavy'" in perf_output:
            print("[bench.py] Skipping unsupported workload: read_heavy")
            return False
        print("[bench.py] perf invocation failed; retrying benchmark without perf stat.")
        return run_command(command, allow_unsupported_read_heavy=True)


def generate_plots(results_path: Path) -> None:
    """Generate one PNG per workload from the accumulated CSV rows."""
    print(f"[bench.py] Loading results from {results_path}")
    df = pd.read_csv(results_path)

    PLOTS_DIR.mkdir(exist_ok=True)

    for workload in WORKLOADS:
        subset = df[df["workload"] == workload]
        if subset.empty:
            print(f"[bench.py] No rows for workload={workload}; skipping plot generation.")
            continue

        datasets = sorted(subset["dataset"].unique(), key=DATASETS.index)
        tables = sorted(subset["table"].unique())
        x_positions = list(range(len(datasets)))
        width = 0.8 / max(len(tables), 1)

        fig, axes = plt.subplots(1, 2, figsize=(16, 6), constrained_layout=True)

        for axis, metric, title in [
            (axes[0], "insert_ns_per_op", "Insert ns/op"),
            (axes[1], "find_ns_per_op", "Find ns/op"),
        ]:
            for index, table in enumerate(tables):
                table_subset = subset[subset["table"] == table]
                values = []
                for dataset in datasets:
                    dataset_subset = table_subset[table_subset["dataset"] == dataset]
                    values.append(
                        float(dataset_subset.iloc[-1][metric]) if not dataset_subset.empty else 0.0
                    )

                offsets = [
                    x + (index - (len(tables) - 1) / 2) * width for x in x_positions
                ]
                axis.bar(offsets, values, width=width, label=table)

            axis.set_title(f"{workload}: {title}")
            axis.set_xlabel("dataset")
            axis.set_ylabel("ns per operation")
            axis.set_xticks(x_positions)
            axis.set_xticklabels(datasets, rotation=20)

        axes[1].legend(title="table", bbox_to_anchor=(1.02, 1.0), loc="upper left")

        output_path = PLOTS_DIR / f"{workload}.png"
        fig.savefig(output_path, dpi=150)
        plt.close(fig)
        print(f"[bench.py] Saved plot: {output_path}")

        # Generate space efficiency plot
        fig, axis = plt.subplots(figsize=(8, 6), constrained_layout=True)
        metric = "bytes_per_element"
        title = "Space Efficiency"

        for index, table in enumerate(tables):
            table_subset = subset[subset["table"] == table]
            values = []
            for dataset in datasets:
                dataset_subset = table_subset[table_subset["dataset"] == dataset]
                values.append(
                    float(dataset_subset.iloc[-1][metric]) if not dataset_subset.empty else 0.0
                )

            offsets = [
                x + (index - (len(tables) - 1) / 2) * width for x in x_positions
            ]
            axis.bar(offsets, values, width=width, label=table)

        axis.set_title(f"{workload}: {title}")
        axis.set_xlabel("dataset")
        axis.set_ylabel("bytes per element")
        axis.set_xticks(x_positions)
        axis.set_xticklabels(datasets, rotation=20)
        axis.legend(title="table", bbox_to_anchor=(1.02, 1.0), loc="upper left")

        output_path = PLOTS_DIR / f"space_{workload}.png"
        fig.savefig(output_path, dpi=150)
        plt.close(fig)
        print(f"[bench.py] Saved plot: {output_path}")


def main() -> None:
    """Run the benchmark matrix and produce plots from the resulting CSV."""
    ensure_datasets()
    build_release()

    if RESULTS_PATH.exists():
        starting_size = RESULTS_PATH.stat().st_size
    else:
        starting_size = 0

    for dataset in DATASETS:
        for workload in WORKLOADS:
            size = dataset_size(dataset)
            run_benchmark(dataset, workload, size)

    if RESULTS_PATH.exists():
        ending_size = RESULTS_PATH.stat().st_size
        print(
            f"[bench.py] results.csv size: before={starting_size} bytes, after={ending_size} bytes"
        )
    else:
        raise FileNotFoundError("results.csv was not created by benchmark runs")

    generate_plots(RESULTS_PATH)


if __name__ == "__main__":
    main()
