"""
Benchmark for risk_sampler.simulate -- prints a single JSON line.

Run with:  python bench_risk_sampler.py

The benchmark uses `timeit.repeat` which runs the target function multiple
times in a tight loop, then reports the *minimum* batch time. We convert
that to a mean per-call nanosecond figure so it's directly comparable to
Rust's criterion output.
"""

from __future__ import annotations

import json
import timeit

from risk_sampler import RiskEvent, simulate

# Match the portfolio from the Rust main.rs so the benchmark is comparable.
EVENTS: list[RiskEvent] = [
    RiskEvent(name="Cyber attack", probability=0.05, max_loss=500_000.0),
    RiskEvent(name="Server outage", probability=0.15, max_loss=50_000.0),
    RiskEvent(name="Supply chain delay", probability=0.20, max_loss=25_000.0),
    RiskEvent(name="Regulatory fine", probability=0.02, max_loss=1_000_000.0),
]

TRIALS: int = 100_000
SEED: int = 42
ITERATIONS: int = 100  # how many times timeit repeats the call


def _target() -> None:
    """Single invocation of simulate, used as the timeit target."""
    simulate(EVENTS, TRIALS, SEED)


def main() -> None:
    # `timeit.repeat` returns a list of total-seconds for each repeat batch.
    # With number=1 each entry is one call, and we take the best (minimum)
    # to reduce noise from OS scheduling jitter.
    times: list[float] = timeit.repeat(stmt=_target, number=1, repeat=ITERATIONS)

    best_s: float = min(times)
    mean_s: float = sum(times) / len(times)
    mean_ns: int = int(mean_s * 1_000_000_000)
    summary_ms: float = round(mean_s * 1_000, 1)

    result: dict = {
        "ok": True,
        "mean_ns": mean_ns,
        "summary": f"{TRIALS:,} trials in {summary_ms}ms",
        "iterations": ITERATIONS,
    }

    print(json.dumps(result))


if __name__ == "__main__":
    main()
