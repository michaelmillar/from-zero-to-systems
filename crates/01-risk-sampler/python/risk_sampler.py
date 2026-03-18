"""
Risk Sampler - Monte Carlo simulation of risk events.

Mirrors the Rust reference implementation: for each trial, iterate every event,
check whether it fires (probability), sample a uniform loss in [0, max_loss),
and accumulate. VaR95 is the sorted-losses value at the 95th percentile index.
"""

from __future__ import annotations

import random

# `dataclasses` let you declare structured data without writing __init__ / __repr__
# boilerplate. The @dataclass decorator auto-generates those methods from the
# annotated fields, similar to Rust's derive macros.
from dataclasses import dataclass


@dataclass
class RiskEvent:
    """A single risk event with a firing probability and maximum loss."""

    name: str
    probability: float  # 0.0 = never fires, 1.0 = always fires
    max_loss: float  # upper bound of the uniform loss distribution


@dataclass
class SimulationResult:
    """Aggregate statistics produced by a Monte Carlo run."""

    trials: int
    occurrences: int  # total number of times *any* event fired across all trials
    total_loss: float
    mean_loss_per_trial: float
    max_observed_loss: float
    var_95: float  # Value-at-Risk at the 95th percentile


def simulate(events: list[RiskEvent], trials: int, seed: int) -> SimulationResult:
    """Run a Monte Carlo simulation over a portfolio of risk events.

    The algorithm is deliberately kept close to the Rust version so you can
    compare the two side by side.

    Parameters
    ----------
    events : list[RiskEvent]
        Portfolio of independent risk events.
    trials : int
        Number of Monte Carlo trials to run.
    seed : int
        Seed for the PRNG. Using `random.Random(seed)` gives a *local*
        generator that won't interfere with any global random state, and
        makes every run deterministic for the same seed.

    Returns
    -------
    SimulationResult
    """

    # `random.Random(seed)` creates an independent Mersenne Twister instance.
    # This is the Python idiom for deterministic, isolated seeding -- it avoids
    # mutating the module-level RNG that other code might depend on.
    rng: random.Random = random.Random(seed)

    occurrences: int = 0
    total_loss: float = 0.0
    max_observed: float = 0.0

    # Pre-allocate a list for per-trial losses (we need them sorted later for VaR).
    # A list comprehension is the Pythonic way to build a list in one expression,
    # but here we append inside the loop because we also need the running totals.
    losses: list[float] = []

    for _ in range(trials):
        trial_loss: float = 0.0

        for event in events:
            # rng.random() returns a float in [0.0, 1.0), matching Rust's
            # rng.gen::<f64>() which also produces [0, 1).
            if rng.random() < event.probability:
                loss: float = rng.random() * event.max_loss
                trial_loss += loss
                occurrences += 1

        total_loss += trial_loss
        if trial_loss > max_observed:
            max_observed = trial_loss
        losses.append(trial_loss)

    # `sorted()` returns a new list; it uses Timsort (O(n log n), stable).
    # The Rust version uses Vec::sort_by with partial_cmp -- same idea.
    sorted_losses: list[float] = sorted(losses)

    # 95th percentile index: truncate the float to int (same as Rust `as usize`).
    var_95_idx: int = int(trials * 0.95)
    var_95: float = sorted_losses[var_95_idx] if var_95_idx < len(sorted_losses) else 0.0

    return SimulationResult(
        trials=trials,
        occurrences=occurrences,
        total_loss=total_loss,
        mean_loss_per_trial=total_loss / trials,
        max_observed_loss=max_observed,
        var_95=var_95,
    )
