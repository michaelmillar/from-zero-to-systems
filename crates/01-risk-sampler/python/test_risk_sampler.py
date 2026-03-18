"""
Tests for risk_sampler -- mirrors the four Rust tests exactly.

Run with:  pytest test_risk_sampler.py -v
"""

from risk_sampler import RiskEvent, SimulationResult, simulate


def test_zero_probability_event_never_occurs() -> None:
    """An event with probability 0.0 should never fire, producing zero loss."""
    events: list[RiskEvent] = [
        RiskEvent(name="never", probability=0.0, max_loss=1_000_000.0),
    ]
    result: SimulationResult = simulate(events, trials=10_000, seed=42)

    assert result.occurrences == 0
    assert result.total_loss == 0.0


def test_certain_event_always_occurs() -> None:
    """An event with probability 1.0 should fire every single trial."""
    events: list[RiskEvent] = [
        RiskEvent(name="always", probability=1.0, max_loss=100.0),
    ]
    result: SimulationResult = simulate(events, trials=1_000, seed=42)

    assert result.occurrences == 1_000
    assert result.total_loss > 0.0


def test_var_95_is_not_greater_than_max_possible_loss() -> None:
    """VaR at the 95th percentile must stay within the single-event max loss."""
    events: list[RiskEvent] = [
        RiskEvent(name="flood", probability=0.1, max_loss=50_000.0),
    ]
    result: SimulationResult = simulate(events, trials=100_000, seed=7)

    assert result.var_95 <= 50_000.0


def test_mean_loss_is_consistent_with_probability() -> None:
    """Over many trials the mean loss should converge to prob * max_loss / 2.

    For a single event firing with probability `p` and a uniform loss in
    [0, max_loss), the expected loss per trial is p * max_loss / 2.
    With 500k trials the sample mean should sit within 5% of that expectation.
    """
    prob: float = 0.2
    max_loss: float = 1_000.0
    events: list[RiskEvent] = [
        RiskEvent(name="outage", probability=prob, max_loss=max_loss),
    ]
    result: SimulationResult = simulate(events, trials=500_000, seed=99)

    expected: float = prob * max_loss / 2.0
    tolerance: float = expected * 0.05

    assert abs(result.mean_loss_per_trial - expected) < tolerance, (
        f"mean {result.mean_loss_per_trial:.2f} not within 5% of expected {expected:.2f}"
    )
