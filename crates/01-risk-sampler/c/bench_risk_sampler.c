/*
 * bench_risk_sampler.c
 *
 * Benchmark harness for the risk-sampler Monte Carlo simulation.
 *
 * Runs the simulation `ITERATIONS` times, measures wall-clock time with
 * clock_gettime(CLOCK_MONOTONIC), and emits a single JSON line:
 *
 *   {"ok": true, "mean_ns": <value>, "summary": "<text>", "iterations": 100}
 *
 * Compile with -O2 for a representative measurement.
 */

#define _DEFAULT_SOURCE
#include "risk_sampler.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdint.h>
#include <time.h>

#define ITERATIONS 100
#define TRIALS     100000
#define SEED       42

int main(void)
{
    /* Build the same event portfolio used by the Rust main binary. */
    RiskEvent events[] = {
        { "Cyber attack",         0.05, 500000.0  },
        { "Server outage",        0.15,  50000.0  },
        { "Supply chain delay",   0.20,  25000.0  },
        { "Regulatory fine",      0.02, 1000000.0 },
    };
    uint64_t event_count = sizeof(events) / sizeof(events[0]);

    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);

    for (int i = 0; i < ITERATIONS; i++) {
        /* volatile to prevent the compiler from optimising away the call */
        volatile SimulationResult r = simulate(events, event_count, TRIALS, SEED);
        (void)r;
    }

    clock_gettime(CLOCK_MONOTONIC, &t1);

    int64_t elapsed_ns = (int64_t)(t1.tv_sec - t0.tv_sec) * 1000000000LL
                       + (int64_t)(t1.tv_nsec - t0.tv_nsec);
    int64_t mean_ns    = elapsed_ns / ITERATIONS;

    printf("{\"ok\": true, \"mean_ns\": %" PRId64 ", "
           "\"summary\": \"%d iterations of %d trials (seed %d) in %" PRId64 " ms, "
           "mean %" PRId64 " ns/iter\", \"iterations\": %d}\n",
           mean_ns,
           ITERATIONS, TRIALS, SEED,
           elapsed_ns / 1000000,
           mean_ns,
           ITERATIONS);

    return 0;
}
