/*
 * risk_sampler.c
 *
 * Monte Carlo risk simulation in plain C11.
 *
 * Teaching points covered in this file
 * -------------------------------------
 *  1. Manual malloc / free    (no RAII, no Drop, no garbage collector)
 *  2. Function pointers       (qsort comparator)
 *  3. No generics             (concrete types everywhere)
 *  4. rand_r for thread-safe  (unlike rand/srand which use global state)
 */

#define _DEFAULT_SOURCE
#include "risk_sampler.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>

/* ------------------------------------------------------------------ */
/*  PRNG helpers                                                       */
/* ------------------------------------------------------------------ */

/*
 * Teaching point: rand_r and thread safety
 * -----------------------------------------
 * The standard rand()/srand() pair shares a single global seed, which makes
 * it neither thread-safe nor reproducible when multiple simulations run in
 * parallel. rand_r() takes a pointer to caller-owned state so each call-site
 * controls its own sequence.
 *
 * Note that rand_r is POSIX, not ISO C. For maximum portability we wrap it
 * behind a thin helper so the rest of the code does not care about the
 * underlying generator. On platforms without rand_r you could swap in an
 * LCG or xorshift here without touching simulate().
 */

/* Return a uniform double in [0.0, 1.0). */
static double rand_double(unsigned int *state)
{
    /*
     * rand_r returns [0, RAND_MAX]. Dividing by (RAND_MAX + 1.0) gives
     * [0.0, 1.0) which matches Rust's rng.gen::<f64>() distribution.
     */
    int r = rand_r(state);
    return (double)r / ((double)RAND_MAX + 1.0);
}

/* ------------------------------------------------------------------ */
/*  qsort comparator                                                   */
/* ------------------------------------------------------------------ */

/*
 * Teaching point: function pointers for qsort
 * ---------------------------------------------
 * Rust's sort_by takes a closure (an anonymous function with captured env).
 * C's qsort takes a raw function pointer: int (*)(const void *, const void *).
 *
 * Because the signature uses `const void *`, we must cast back to the
 * concrete type inside the comparator. There is no type checking here;
 * passing the wrong comparator compiles fine but produces nonsense at
 * runtime. Rust's generics prevent this class of bug entirely.
 */
static int compare_doubles(const void *a, const void *b)
{
    double da = *(const double *)a;
    double db = *(const double *)b;

    if (da < db) return -1;
    if (da > db) return  1;
    return 0;
}

/* ------------------------------------------------------------------ */
/*  Core simulation                                                    */
/* ------------------------------------------------------------------ */

SimulationResult simulate(const RiskEvent *events,
                          uint64_t         event_count,
                          uint64_t         trials,
                          uint64_t         seed)
{
    /*
     * Teaching point: manual malloc / free
     * -------------------------------------
     * In Rust: `let mut losses: Vec<f64> = Vec::with_capacity(trials);`
     * The Vec owns its heap buffer and frees it when it goes out of scope.
     *
     * In C we must:
     *   1. Allocate with malloc (or calloc).
     *   2. Check the return value for NULL.
     *   3. Free the buffer manually before every return path.
     *
     * Forgetting step 3 is a memory leak. Using `losses` after free is
     * undefined behaviour ("use after free"). Rust's ownership system
     * makes both errors compile-time failures.
     */
    double *losses = malloc(sizeof(double) * (size_t)trials);
    if (!losses) {
        fprintf(stderr, "risk_sampler: malloc failed for %" PRIu64 " trials\n",
                trials);
        abort();
    }

    unsigned int rng_state = (unsigned int)seed;

    uint64_t occurrences   = 0;
    double   total_loss    = 0.0;
    double   max_observed  = 0.0;

    for (uint64_t t = 0; t < trials; t++) {
        double trial_loss = 0.0;

        for (uint64_t e = 0; e < event_count; e++) {
            if (rand_double(&rng_state) < events[e].probability) {
                double loss = rand_double(&rng_state) * events[e].max_loss;
                trial_loss += loss;
                occurrences++;
            }
        }

        total_loss += trial_loss;
        if (trial_loss > max_observed) {
            max_observed = trial_loss;
        }
        losses[t] = trial_loss;
    }

    /*
     * Teaching point: no generics in qsort
     * --------------------------------------
     * Rust:  losses.sort_by(|a, b| a.partial_cmp(b).unwrap());
     * C:     qsort(losses, trials, sizeof(double), compare_doubles);
     *
     * qsort is "generic" only through void pointers and a function pointer
     * comparator. The compiler cannot verify that compare_doubles actually
     * compares doubles; you could accidentally pass a comparator for ints
     * and the programme would silently produce wrong results.
     */
    qsort(losses, (size_t)trials, sizeof(double), compare_doubles);

    size_t var_95_idx = (size_t)((double)trials * 0.95);
    double var_95     = 0.0;
    if (var_95_idx < (size_t)trials) {
        var_95 = losses[var_95_idx];
    }

    /* Free the heap buffer. After this line, `losses` is dangling. */
    free(losses);

    SimulationResult result;
    result.trials             = trials;
    result.occurrences        = occurrences;
    result.total_loss         = total_loss;
    result.mean_loss_per_trial = total_loss / (double)trials;
    result.max_observed_loss  = max_observed;
    result.var_95             = var_95;

    return result;
}
