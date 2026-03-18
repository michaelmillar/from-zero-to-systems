CC      = gcc
CFLAGS  = -Wall -Wextra -std=c11
LDFLAGS = -lm

# Bench builds with optimisations so timings are representative.
BENCH_CFLAGS = $(CFLAGS) -O2

.PHONY: test bench clean

# ---- build targets ------------------------------------------------

test_risk_sampler: test_risk_sampler.c risk_sampler.c risk_sampler.h
	$(CC) $(CFLAGS) -o $@ test_risk_sampler.c risk_sampler.c $(LDFLAGS)

bench_risk_sampler: bench_risk_sampler.c risk_sampler.c risk_sampler.h
	$(CC) $(BENCH_CFLAGS) -o $@ bench_risk_sampler.c risk_sampler.c $(LDFLAGS)

# ---- phony targets ------------------------------------------------

test: test_risk_sampler
	./test_risk_sampler

bench: bench_risk_sampler
	./bench_risk_sampler

clean:
	rm -f test_risk_sampler bench_risk_sampler
