#!/usr/bin/env python3
"""
Klar AI Correctness Benchmark Runner

Runs benchmark tasks against the Klar compiler and reports correctness.

Usage:
    python run.py                          # Run all tasks
    python run.py --category basics        # Run one category
    python run.py --task 001               # Run one task
    python run.py --report                 # Generate report
"""

import os
import sys
import yaml
import subprocess
import json
import time
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional

TASKS_DIR = Path(__file__).parent.parent / "tasks"
RESULTS_DIR = Path(__file__).parent.parent / "results"

@dataclass
class TaskResult:
    task_id: str
    title: str
    category: str
    passed: bool
    compile_ok: bool
    error: Optional[str] = None
    duration_ms: float = 0.0

@dataclass
class BenchmarkReport:
    total: int = 0
    passed: int = 0
    failed: int = 0
    compile_errors: int = 0
    runtime_errors: int = 0
    results: list = field(default_factory=list)
    duration_ms: float = 0.0

    @property
    def correctness(self) -> float:
        return (self.passed / self.total * 100) if self.total > 0 else 0.0


def load_tasks(category: Optional[str] = None, task_id: Optional[str] = None) -> list[dict]:
    """Load benchmark tasks from YAML files."""
    tasks = []
    for f in sorted(TASKS_DIR.glob("*.yaml")):
        with open(f) as fh:
            task = yaml.safe_load(fh)
            if task_id and task["id"] != task_id:
                continue
            if category and task.get("category") != category:
                continue
            tasks.append(task)
    return tasks


def run_task(task: dict) -> TaskResult:
    """Run a single benchmark task using the solution + test."""
    start = time.time()
    task_id = task["id"]
    title = task["title"]
    category = task.get("category", "unknown")

    # Combine solution and test into a single Klar program
    source = task.get("solution", "") + "\n\n" + task.get("test", "")

    # Write to temp file
    tmp_file = f"/tmp/klar_bench_{task_id}.klar"
    with open(tmp_file, "w") as f:
        f.write(source)

    # Run klar test
    try:
        result = subprocess.run(
            ["klar", "test", tmp_file],
            capture_output=True, text=True, timeout=10
        )
        duration = (time.time() - start) * 1000

        if result.returncode == 0:
            return TaskResult(task_id, title, category, True, True, duration_ms=duration)
        else:
            error = result.stderr.strip() or result.stdout.strip()
            is_compile = "parse error" in error or "type error" in error
            return TaskResult(
                task_id, title, category, False,
                compile_ok=not is_compile,
                error=error[:200],
                duration_ms=duration
            )
    except subprocess.TimeoutExpired:
        return TaskResult(task_id, title, category, False, True, error="timeout", duration_ms=10000)
    except FileNotFoundError:
        return TaskResult(task_id, title, category, False, False, error="klar not found in PATH")


def run_benchmark(category=None, task_id=None) -> BenchmarkReport:
    """Run the full benchmark suite."""
    tasks = load_tasks(category, task_id)
    report = BenchmarkReport()
    report.total = len(tasks)

    start = time.time()

    for task in tasks:
        result = run_task(task)
        report.results.append(result)

        status = "\033[32m✓\033[0m" if result.passed else "\033[31m✗\033[0m"
        print(f"  {status} [{result.task_id}] {result.title} ({result.duration_ms:.0f}ms)")

        if result.passed:
            report.passed += 1
        else:
            report.failed += 1
            if not result.compile_ok:
                report.compile_errors += 1
            else:
                report.runtime_errors += 1

    report.duration_ms = (time.time() - start) * 1000
    return report


def print_report(report: BenchmarkReport):
    """Print a formatted benchmark report."""
    print()
    print("=" * 60)
    print(f"  Klar AI Correctness Benchmark")
    print("=" * 60)
    print(f"  Total tasks:      {report.total}")
    print(f"  Passed:           \033[32m{report.passed}\033[0m")
    print(f"  Failed:           \033[31m{report.failed}\033[0m")
    print(f"    Compile errors: {report.compile_errors}")
    print(f"    Runtime errors: {report.runtime_errors}")
    print(f"  Correctness:      \033[1m{report.correctness:.1f}%\033[0m")
    print(f"  Duration:         {report.duration_ms:.0f}ms")
    print("=" * 60)

    # By category
    categories = {}
    for r in report.results:
        cat = r.category
        if cat not in categories:
            categories[cat] = {"total": 0, "passed": 0}
        categories[cat]["total"] += 1
        if r.passed:
            categories[cat]["passed"] += 1

    if categories:
        print()
        print("  By category:")
        for cat, data in sorted(categories.items()):
            pct = data["passed"] / data["total"] * 100
            print(f"    {cat:<20} {data['passed']}/{data['total']} ({pct:.0f}%)")


def save_report(report: BenchmarkReport):
    """Save report as JSON."""
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    out_file = RESULTS_DIR / f"benchmark_{timestamp}.json"

    data = {
        "timestamp": timestamp,
        "total": report.total,
        "passed": report.passed,
        "failed": report.failed,
        "correctness": report.correctness,
        "duration_ms": report.duration_ms,
        "results": [
            {
                "task_id": r.task_id,
                "title": r.title,
                "category": r.category,
                "passed": r.passed,
                "compile_ok": r.compile_ok,
                "error": r.error,
                "duration_ms": r.duration_ms,
            }
            for r in report.results
        ],
    }

    with open(out_file, "w") as f:
        json.dump(data, f, indent=2)
    print(f"\n  Report saved: {out_file}")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Klar Benchmark Runner")
    parser.add_argument("--category", help="Run only this category")
    parser.add_argument("--task", help="Run only this task ID")
    parser.add_argument("--save", action="store_true", help="Save results to JSON")
    args = parser.parse_args()

    print()
    print("  Running Klar benchmark...")
    print()

    report = run_benchmark(category=args.category, task_id=args.task)
    print_report(report)

    if args.save:
        save_report(report)
