/*
 * risk_sampler.h
 *
 * Header for the risk-sampler Monte Carlo simulation.
 *
 * Teaching points
 * ---------------
 * In C there are no generics. Every struct is concrete and every function
 * signature names its exact types. Compare with Rust where `simulate` takes
 * `&[RiskEvent]` (a fat pointer carrying length). Here we pass a raw pointer
 * plus a separate `count` because C arrays decay to pointers and lose their
 * size information.
 *
 * Structs are plain value types. There is no String; we use a `const char *`
 * that points to a string literal or caller-owned buffer. The caller is
 * responsible for the lifetime of that pointer.
 */

#ifndef RISK_SAMPLER_H
#define RISK_SAMPLER_H

#include <stdint.h>

/* ------------------------------------------------------------------ */
/*  Data types                                                         */
/* ------------------------------------------------------------------ */

typedef struct {
    const char *name;     /* borrowed pointer, not owned */
    double probability;   /* 0.0 .. 1.0                  */
    double max_loss;      /* upper bound on loss amount   */
} RiskEvent;

typedef struct {
    uint64_t trials;
    uint64_t occurrences;
    double   total_loss;
    double   mean_loss_per_trial;
    double   max_observed_loss;
    double   var_95;
} SimulationResult;

/* ------------------------------------------------------------------ */
/*  Public API                                                         */
/* ------------------------------------------------------------------ */

/*
 * Run `trials` Monte Carlo iterations over `event_count` risk events.
 *
 * `seed` initialises a deterministic PRNG so results are reproducible.
 *
 * Internally this malloc's a losses array of `trials` doubles and frees it
 * before returning. If allocation fails the programme aborts.
 *
 * Teaching point: manual malloc/free
 * -----------------------------------
 * Rust's Vec<f64> handles heap allocation and deallocation automatically via
 * Drop. In C we must allocate with malloc, check for NULL, and remember to
 * free. Missing the free is a memory leak; using the pointer after free is
 * undefined behaviour.
 */
SimulationResult simulate(const RiskEvent *events,
                          uint64_t         event_count,
                          uint64_t         trials,
                          uint64_t         seed);

#endif /* RISK_SAMPLER_H */
