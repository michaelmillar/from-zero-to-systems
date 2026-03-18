/*
 * test_risk_sampler.c
 *
 * Minimal test harness for the risk-sampler C implementation.
 *
 * Output format matches the Rust convention:
 *     test <name> ... ok
 *     test <name> ... FAILED
 *
 * Returns 0 when all tests pass, 1 when any test fails.
 *
 * Teaching point: there is no built-in test framework in C
 * ---------------------------------------------------------
 * Rust has `#[test]` and `cargo test` out of the box. In C you either
 * roll your own harness (as below), use a micro-framework like greatest.h
 * or munit, or pull in a heavier library like Check or cmocka.
 *
 * The pattern here is deliberately simple: each test is a function that
 * returns 1 on success, 0 on failure. main() collects results and
 * produces the human-readable report.
 */

#include "risk_sampler.h"

#include <math.h>
#include <stdio.h>

/* ------------------------------------------------------------------ */
/*  Helpers                                                            */
/* ------------------------------------------------------------------ */

static int report(const char *name, int passed)
{
    printf("test %s ... %s\n", name, passed ? "ok" : "FAILED");
    return passed;
}

/* ------------------------------------------------------------------ */
/*  Tests                                                              */
/* ------------------------------------------------------------------ */

/*
 * A zero-probability event should never fire regardless of the seed or
 * number of trials.
 */
static int test_zero_probability(void)
{
    RiskEvent events[] = {
        { .name = "never", .probability = 0.0, .max_loss = 1000000.0 }
    };

    SimulationResult r = simulate(events, 1, 10000, 42);

    int ok = (r.occurrences == 0) && (r.total_loss == 0.0);
    return report("zero_probability_event_never_occurs", ok);
}

/*
 * A probability-1.0 event must fire on every single trial.
 */
static int test_certain_event(void)
{
    RiskEvent events[] = {
        { .name = "always", .probability = 1.0, .max_loss = 100.0 }
    };

    SimulationResult r = simulate(events, 1, 1000, 42);

    int ok = (r.occurrences == 1000) && (r.total_loss > 0.0);
    return report("certain_event_always_occurs", ok);
}

/*
 * The 95th-percentile Value at Risk can never exceed the maximum
 * possible loss from a single event.
 */
static int test_var_95_bounded(void)
{
    RiskEvent events[] = {
        { .name = "flood", .probability = 0.1, .max_loss = 50000.0 }
    };

    SimulationResult r = simulate(events, 1, 100000, 7);

    int ok = (r.var_95 <= 50000.0);
    return report("var_95_not_greater_than_max_possible_loss", ok);
}

/*
 * Over many trials the mean loss should converge to the theoretical
 * expectation: probability * max_loss / 2.
 *
 * We allow 5% relative tolerance, same as the Rust test.
 */
static int test_mean_loss_convergence(void)
{
    double prob     = 0.2;
    double max_loss = 1000.0;

    RiskEvent events[] = {
        { .name = "outage", .probability = prob, .max_loss = max_loss }
    };

    SimulationResult r = simulate(events, 1, 500000, 99);

    double expected  = prob * max_loss / 2.0;
    double tolerance = expected * 0.05;
    double diff      = fabs(r.mean_loss_per_trial - expected);

    int ok = (diff < tolerance);
    if (!ok) {
        fprintf(stderr,
                "  mean %.2f not within 5%% of expected %.2f (diff %.2f)\n",
                r.mean_loss_per_trial, expected, diff);
    }
    return report("mean_loss_consistent_with_probability", ok);
}

/* ------------------------------------------------------------------ */
/*  Main                                                               */
/* ------------------------------------------------------------------ */

int main(void)
{
    printf("\nrunning 4 tests\n");

    int passed = 0;
    passed += test_zero_probability();
    passed += test_certain_event();
    passed += test_var_95_bounded();
    passed += test_mean_loss_convergence();

    printf("\ntest result: %s. %d passed; %d failed\n\n",
           passed == 4 ? "ok" : "FAILED",
           passed, 4 - passed);

    return (passed == 4) ? 0 : 1;
}
