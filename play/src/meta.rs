pub struct DocLink {
    pub label: &'static str,
    pub url:   &'static str,
}

pub struct TestHints {
    /// Substring matched against the leaf test name.
    pub test_name: &'static str,
    /// Up to 3 hints, revealed one at a time with [h].
    pub hints: &'static [&'static str],
}

pub struct CrateMeta {
    pub package:  &'static str,
    pub display:  &'static str,
    pub concepts: &'static [&'static str],
    pub docs:     &'static [DocLink],
    pub tests:    &'static [TestHints],
}

pub const CRATES: &[CrateMeta] = &[
    // ------------------------------------------------------------------
    CrateMeta {
        package: "risk-sampler", display: "01 · risk-sampler",
        concepts: &[
            "Monte Carlo simulation",
            "Loss distribution sampling",
            "Value at Risk (VaR) - 95th percentile",
            "Seeded deterministic RNG",
        ],
        docs: &[
            DocLink { label: "rand::SeedableRng", url: "https://docs.rs/rand/latest/rand/trait.SeedableRng.html" },
            DocLink { label: "Monte Carlo method", url: "https://en.wikipedia.org/wiki/Monte_Carlo_method" },
        ],
        tests: &[
            TestHints {
                test_name: "zero_probability",
                hints: &[
                    "If probability == 0.0, the event never fires. Return result with zero occurrences and zero total_loss.",
                    "Loop over events: if rng.gen::<f64>() < event.probability the event fires. With prob=0, it never will.",
                ],
            },
            TestHints {
                test_name: "certain_event",
                hints: &[
                    "If probability == 1.0, every trial fires. occurrences should equal trials.",
                    "Each firing samples a uniform loss in [0, max_loss]: rng.gen::<f64>() * max_loss.",
                    "Accumulate total_loss across trials. mean_loss = total_loss / trials as f64.",
                ],
            },
            TestHints {
                test_name: "var_95",
                hints: &[
                    "Collect all per-trial loss totals, sort them, then read the 95th percentile.",
                    "var_95 = sorted_losses[(0.95 * trials as f64) as usize]. var_95 <= max possible loss.",
                ],
            },
            TestHints {
                test_name: "mean_loss",
                hints: &[
                    "Expected loss per trial = prob * max_loss / 2 (uniform distribution mean).",
                    "mean_loss_per_trial = total_loss / trials as f64. With enough trials it converges.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "probability-engine", display: "02 · probability-engine",
        concepts: &[
            "Bernoulli distribution: single yes/no trial",
            "Beta distribution: conjugate prior for Bernoulli",
            "Bayesian update rule: posterior = prior + data",
            "Gamma sampling (Marsaglia-Tsang method, already given)",
        ],
        docs: &[
            DocLink { label: "Beta distribution", url: "https://en.wikipedia.org/wiki/Beta_distribution" },
            DocLink { label: "Conjugate prior", url: "https://en.wikipedia.org/wiki/Conjugate_prior" },
        ],
        tests: &[
            TestHints {
                test_name: "bernoulli_mean",
                hints: &["Bernoulli(p) mean = p. Just return self.p."],
            },
            TestHints {
                test_name: "bernoulli_variance",
                hints: &["Bernoulli(p) variance = p * (1 - p)."],
            },
            TestHints {
                test_name: "beta_mean",
                hints: &["Beta(alpha, beta) mean = alpha / (alpha + beta)."],
            },
            TestHints {
                test_name: "bayesian_update",
                hints: &[
                    "Bayesian update: posterior = Beta(alpha + successes, beta + failures).",
                    "Beta(1,1) + 8 successes + 2 failures -> Beta(9, 3). mean = 9/12 = 0.75.",
                ],
            },
            TestHints {
                test_name: "bernoulli_samples",
                hints: &["Sample: if rng.gen::<f64>() < self.p return 1.0 else 0.0."],
            },
            TestHints {
                test_name: "beta_sample",
                hints: &[
                    "Beta sample: x = sample_gamma(alpha, rng); y = sample_gamma(beta, rng); x / (x + y).",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "monte-carlo", display: "03 · monte-carlo",
        concepts: &[
            "European call option pricing",
            "Geometric Brownian motion: S_T = S_0 * exp((r-0.5σ²)T + σ√T·Z)",
            "Black-Scholes analytical formula (for validation)",
            "Value at Risk: loss at given confidence percentile",
        ],
        docs: &[
            DocLink { label: "Black-Scholes model", url: "https://en.wikipedia.org/wiki/Black%E2%80%93Scholes_model" },
            DocLink { label: "Geometric Brownian motion", url: "https://en.wikipedia.org/wiki/Geometric_Brownian_motion" },
        ],
        tests: &[
            TestHints {
                test_name: "mc_option_price",
                hints: &[
                    "Simulate: S_T = spot * exp((rate - 0.5*vol²)*expiry + vol*sqrt(expiry)*Z) where Z ~ N(0,1).",
                    "Payoff = (S_T - strike).max(0.0). Price = exp(-rate*expiry) * mean(payoffs).",
                    "Use rand_distr::Normal::new(0.0, 1.0) and rayon's into_par_iter() for parallelism.",
                ],
            },
            TestHints {
                test_name: "deep_out_of_money",
                hints: &[
                    "With strike=200, spot=100: S_T rarely exceeds 200. Payoff = max(S_T-200, 0) is near zero.",
                ],
            },
            TestHints {
                test_name: "deep_in_money",
                hints: &[
                    "With spot=200, strike=100: payoff = max(S_T-100, 0) is almost always large (>95).",
                ],
            },
            TestHints {
                test_name: "var_95_is_less",
                hints: &[
                    "Sort returns. VaR at confidence c = -(return at (1-c) percentile). VaR_99 > VaR_95.",
                    "value_at_risk: sort the slice, return -sorted[(1-confidence) * n index] or similar.",
                ],
            },
            TestHints {
                test_name: "var_of_all_gains",
                hints: &[
                    "If all returns are positive, the 95th percentile loss is zero or negative. VaR <= 0.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "distribution-sampler", display: "04 · distribution-sampler",
        concepts: &[
            "Exponential: inverse CDF sampling (-ln(U)/lambda)",
            "Poisson: Knuth's algorithm (product of uniforms)",
            "Weibull: generalises Exponential (k < 1: infant mortality; k > 1: wear-out)",
            "Gamma function (Lanczos approx, already given)",
        ],
        docs: &[
            DocLink { label: "Inverse transform sampling", url: "https://en.wikipedia.org/wiki/Inverse_transform_sampling" },
            DocLink { label: "Poisson distribution", url: "https://en.wikipedia.org/wiki/Poisson_distribution" },
        ],
        tests: &[
            TestHints {
                test_name: "exponential_sample_mean",
                hints: &[
                    "Exponential sample: -rng.gen::<f64>().max(1e-15).ln() / self.lambda.",
                    "mean() = 1.0 / self.lambda. With 200k samples it converges to within 2%.",
                ],
            },
            TestHints {
                test_name: "poisson_sample_mean",
                hints: &[
                    "Knuth: let L = (-lambda).exp(); p=1.0; k=0. Loop: p *= rng.gen::<f64>(); k += 1; until p < L. Return (k-1) as f64.",
                ],
            },
            TestHints {
                test_name: "weibull_shape1",
                hints: &[
                    "Weibull mean = scale * gamma(1 + 1/shape). When shape=1: gamma(2)=1, mean=scale.",
                    "With shape=1 and scale=2: mean=2. Exponential(lambda=0.5) also has mean=2. They match.",
                ],
            },
            TestHints {
                test_name: "weibull_mean_matches",
                hints: &[
                    "Weibull mean = self.scale * gamma(1.0 + 1.0 / self.shape). Use the provided gamma() function.",
                    "Weibull sample: self.scale * (-rng.gen::<f64>().max(1e-15).ln()).powf(1.0 / self.shape).",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "statistics-core", display: "05 · statistics-core",
        concepts: &[
            "Mean, variance (population N), sample variance (N-1)",
            "Median (sort + midpoint), percentile (linear interpolation)",
            "Skewness (3rd moment), excess kurtosis (4th moment - 3)",
            "IQR outlier detection: [Q1 - 1.5*IQR, Q3 + 1.5*IQR]",
        ],
        docs: &[
            DocLink { label: "Standard deviation", url: "https://en.wikipedia.org/wiki/Standard_deviation" },
            DocLink { label: "Interquartile range", url: "https://en.wikipedia.org/wiki/Interquartile_range" },
        ],
        tests: &[
            TestHints {
                test_name: "mean_of_known",
                hints: &["Return Err(StatsError::Empty) if data is empty. Otherwise sum / len."],
            },
            TestHints {
                test_name: "variance_of_constant",
                hints: &["Population variance = mean of (x - mean)^2. All equal -> all deviations=0."],
            },
            TestHints {
                test_name: "median_even",
                hints: &[
                    "Sort a copy of data. For even length: (sorted[n/2-1] + sorted[n/2]) / 2.0.",
                ],
            },
            TestHints {
                test_name: "percentile_0",
                hints: &[
                    "Sort data. index = p * (n-1) as f64. lower = floor(index). frac = index - lower.",
                    "result = sorted[lower] + frac * (sorted[lower+1] - sorted[lower]). p=0 -> sorted[0], p=1 -> sorted[n-1].",
                ],
            },
            TestHints {
                test_name: "z_scores",
                hints: &["z[i] = (x[i] - mean) / std_dev. Result has mean ~0 and population variance ~1."],
            },
            TestHints {
                test_name: "iqr_outliers",
                hints: &[
                    "IQR = Q3 - Q1. Lower fence = Q1 - 1.5*IQR. Upper fence = Q3 + 1.5*IQR.",
                    "Outliers are values where x < lower_fence || x > upper_fence.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "matrix-math", display: "06 · matrix-math",
        concepts: &[
            "Row-major matrix storage: element (r,c) at index r*cols+c",
            "Transpose: swap rows and columns",
            "Matrix multiplication: result[i][j] = sum_k a[i][k] * b[k][j]",
            "Gaussian elimination with partial pivoting for inverse + determinant",
        ],
        docs: &[
            DocLink { label: "Gaussian elimination", url: "https://en.wikipedia.org/wiki/Gaussian_elimination" },
            DocLink { label: "Matrix multiplication", url: "https://en.wikipedia.org/wiki/Matrix_multiplication" },
        ],
        tests: &[
            TestHints {
                test_name: "identity_times_matrix",
                hints: &[
                    "matmul: return None if self.cols != rhs.rows. result[(i,j)] = sum over k of self[(i,k)] * rhs[(k,j)].",
                ],
            },
            TestHints {
                test_name: "transpose_twice",
                hints: &["transpose: result[(j,i)] = self[(i,j)]. New shape is (self.cols x self.rows)."],
            },
            TestHints {
                test_name: "inverse_of_identity",
                hints: &[
                    "Build augmented matrix [A | I]. Row-reduce to [I | A^-1] using partial pivoting.",
                    "At each step, swap the row with the largest absolute value in the pivot column.",
                    "Track sign from row swaps; determinant = product of pivots (negated per swap).",
                ],
            },
            TestHints {
                test_name: "determinant_of_singular",
                hints: &["If any pivot becomes exactly zero (or near-zero), the matrix is singular. Return None."],
            },
            TestHints {
                test_name: "mul_vec",
                hints: &[
                    "mul_vec: result[i] = sum over k of self[(i,k)] * v[k]. Return None if self.cols != v.len().",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "linear-regression", display: "07 · linear-regression",
        concepts: &[
            "Ordinary Least Squares via normal equations: β = (X'X)⁻¹X'y",
            "Design matrix: prepend a column of 1s for the intercept",
            "R-squared: 1 - SS_res / SS_tot",
            "coefficients[0] = intercept, coefficients[1..] = slopes",
        ],
        docs: &[
            DocLink { label: "OLS regression", url: "https://en.wikipedia.org/wiki/Ordinary_least_squares" },
            DocLink { label: "Normal equations", url: "https://en.wikipedia.org/wiki/Ordinary_least_squares#Matrix/vector_formulation" },
        ],
        tests: &[
            TestHints {
                test_name: "perfect_linear_fit",
                hints: &[
                    "Step 1: build X by prepending a 1.0 column. Each row becomes [1.0, x[0], x[1], ...].",
                    "Step 2: compute X_t*X and X_t*y using matrix_math matmul. Invert X_t*X.",
                    "Step 3: beta = inv * X_t*y. Use mul_vec to get the coefficient vector.",
                ],
            },
            TestHints {
                test_name: "predict_matches",
                hints: &[
                    "predict: dot product of [1.0, x[0], x[1], ...] with model.coefficients.",
                ],
            },
            TestHints {
                test_name: "multi_feature",
                hints: &[
                    "R^2 = 1 - sum(residuals^2) / sum((y - mean_y)^2). Residuals = y - predicted.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "signal-processing", display: "08 · signal-processing",
        concepts: &[
            "Fast Fourier Transform (FFT) via rustfft",
            "Hann window: reduces spectral leakage at signal edges",
            "Frequency bins: bin k corresponds to k * sample_rate / N Hz",
            "RMS = sqrt(mean(x²)). For unit sine: RMS = 1/sqrt(2)",
        ],
        docs: &[
            DocLink { label: "Hann window", url: "https://en.wikipedia.org/wiki/Hann_function" },
            DocLink { label: "rustfft crate", url: "https://docs.rs/rustfft" },
        ],
        tests: &[
            TestHints {
                test_name: "fft_of_pure_sine",
                hints: &[
                    "Pipeline: apply hann_window, convert to Vec<Complex<f64>> with im=0, run FFT, compute magnitudes.",
                    "FftPlanner::new().plan_fft_forward(N).process(&mut buf). magnitude[k] = buf[k].norm() / N as f64.",
                    "dominant_bin = (1..magnitudes.len()).max_by(|a,b| magnitudes[a].partial_cmp(&magnitudes[b])).",
                ],
            },
            TestHints {
                test_name: "hann_window",
                hints: &[
                    "w[i] = 0.5 * (1.0 - (2.0 * PI * i as f64 / (N-1) as f64).cos()). Multiply signal[i] * w[i].",
                ],
            },
            TestHints {
                test_name: "sine_wave",
                hints: &["s[i] = (2.0 * PI * freq_hz * i as f64 / sample_rate).sin()"],
            },
            TestHints {
                test_name: "rms_of_unit_sine",
                hints: &["rms = (signal.iter().map(|x| x*x).sum::<f64>() / N as f64).sqrt()"],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "bit-manipulator", display: "09 · bit-manipulator",
        concepts: &[
            "Bit masks: (1u32 << n) - 1 creates n ones",
            "Extract bits: (value >> offset) & mask",
            "Set/clear/toggle: OR, AND-NOT, XOR with shifted 1",
            "IPv4 header: version in upper 4 bits, IHL in lower 4 bits of byte 0",
        ],
        docs: &[
            DocLink { label: "Bit manipulation", url: "https://en.wikipedia.org/wiki/Bit_manipulation" },
            DocLink { label: "IPv4 header format", url: "https://en.wikipedia.org/wiki/IPv4#Packet_structure" },
        ],
        tests: &[
            TestHints {
                test_name: "extract_bits",
                hints: &[
                    "Mask of len ones: (1u32 << len) - 1. Beware len=32: use .wrapping_shl(len).",
                    "extract_bits = (value >> offset) & mask.",
                ],
            },
            TestHints {
                test_name: "setting_bit",
                hints: &["set_bit = value | (1u32 << bit). clear_bit = value & !(1u32 << bit). toggle = value ^ (1u32 << bit)."],
            },
            TestHints {
                test_name: "count_ones",
                hints: &["value.count_ones() is a Rust built-in method on integer types."],
            },
            TestHints {
                test_name: "rotate_left",
                hints: &["value.rotate_left(n as u32) and value.rotate_right(n as u32) are built-in."],
            },
            TestHints {
                test_name: "swap_bytes",
                hints: &["value.swap_bytes() is a built-in that reverses byte order."],
            },
            TestHints {
                test_name: "ipv4_version",
                hints: &[
                    "Version = upper 4 bits: header_byte0 >> 4. IHL = lower 4 bits: header_byte0 & 0x0F.",
                ],
            },
            TestHints {
                test_name: "ipv4_src_addr",
                hints: &["Source IP is at bytes 12-15 of the header. Return (header[12], header[13], header[14], header[15])."],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "memory-arena", display: "10 · memory-arena",
        concepts: &[
            "Bump allocator: advance a pointer, never free individually",
            "Alignment: pad offset to next multiple of T's alignment",
            "unsafe: cast raw pointer to &mut T",
            "Arena reset: just set offset back to 0 (no deallocation needed)",
        ],
        docs: &[
            DocLink { label: "Arena allocation", url: "https://en.wikipedia.org/wiki/Region-based_memory_management" },
            DocLink { label: "std::alloc::Layout", url: "https://doc.rust-lang.org/std/alloc/struct.Layout.html" },
        ],
        tests: &[
            TestHints {
                test_name: "used_is_zero",
                hints: &["used() returns self.offset. A fresh arena has offset=0."],
            },
            TestHints {
                test_name: "remaining_equals",
                hints: &["remaining() = self.buffer.len() - self.offset."],
            },
            TestHints {
                test_name: "allocating_u64",
                hints: &[
                    "Use std::alloc::Layout::new::<T>() to get size and align.",
                    "Aligned offset: let a = (self.offset + align - 1) & !(align - 1);. Then check a + size <= buffer.len().",
                    "Cast: let ptr = self.buffer.as_mut_ptr().add(a) as *mut T; self.offset = a + size; Some(unsafe { &mut *ptr })",
                ],
            },
            TestHints {
                test_name: "alloc_returns_none",
                hints: &["Return None when aligned_offset + size > self.buffer.len()."],
            },
            TestHints {
                test_name: "after_reset",
                hints: &["reset() sets self.offset = 0. The buffer memory is reused on next alloc."],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "float-inspector", display: "11 · float-inspector",
        concepts: &[
            "IEEE 754 double: [sign:1][exponent:11][mantissa:52]",
            "Exponent bias: stored value = actual + 1023",
            "x.to_bits() gives the raw u64 bit pattern",
            "ULP (Unit in the Last Place): smallest representable step at a given magnitude",
        ],
        docs: &[
            DocLink { label: "IEEE 754 standard", url: "https://en.wikipedia.org/wiki/IEEE_754" },
            DocLink { label: "f64::to_bits()", url: "https://doc.rust-lang.org/std/primitive.f64.html#method.to_bits" },
        ],
        tests: &[
            TestHints {
                test_name: "sign_bit",
                hints: &["bits = x.to_bits(). sign = bits >> 63. 0 = positive, 1 = negative."],
            },
            TestHints {
                test_name: "raw_exponent",
                hints: &[
                    "raw_exponent = (x.to_bits() >> 52) & 0x7FF. For 1.0 this is 1023 (the bias)."],
            },
            TestHints {
                test_name: "actual_exponent",
                hints: &[
                    "actual = raw_exponent as i32 - 1023. Return None for special values (raw=0 or raw=0x7FF).",
                ],
            },
            TestHints {
                test_name: "ulp_distance",
                hints: &[
                    "For same-sign floats: |a.to_bits() as i64 - b.to_bits() as i64| as u64.",
                    "Adjacent representable floats differ by 1 ULP. Identical floats have 0 ULP distance.",
                ],
            },
            TestHints {
                test_name: "cancellation_error",
                hints: &[
                    "(x+1)^2 - x^2 - 2x - 1 is mathematically 0. For large x, (x+1)^2 loses bits. Return the computed (wrong) f64 value.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "mini-vm", display: "12 · mini-vm",
        concepts: &[
            "Stack machine: all operands live on a value stack",
            "Program counter: iterate through instructions in order",
            "Pop order: Sub/Div pop top, then second. Result = second OP top",
            "Halt returns top-of-stack; empty stack at Halt is an error",
        ],
        docs: &[
            DocLink { label: "Stack machine", url: "https://en.wikipedia.org/wiki/Stack_machine" },
            DocLink { label: "Forth language (stack-based)", url: "https://en.wikipedia.org/wiki/Forth_(programming_language)" },
        ],
        tests: &[
            TestHints {
                test_name: "push_then_halt",
                hints: &[
                    "Iterate instructions. Push(n): stack.push(n). Halt: return Ok(stack.last().copied().ok_or(EmptyStack)?)",
                ],
            },
            TestHints {
                test_name: "add_two",
                hints: &[
                    "Pop two values: let top = stack.pop().ok_or(StackUnderflow)?; let second = stack.pop()...;",
                    "Add: push second + top. Sub: push second - top. Mul: push second * top.",
                ],
            },
            TestHints {
                test_name: "div_by_zero",
                hints: &["Div: if top == 0 return Err(DivisionByZero). Else push second / top."],
            },
            TestHints {
                test_name: "program_ending_without_halt",
                hints: &[
                    "When the instruction slice is exhausted (no Halt): return stack.last().copied().ok_or(EmptyStack)",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "consistent-hashing", display: "13 · consistent-hashing",
        concepts: &[
            "Consistent hash ring: nodes and keys share a circular u64 space",
            "Virtual nodes (vnodes): each physical node gets multiple ring positions",
            "BTreeMap for O(log n) clockwise lookup",
            "Minimal key disruption when nodes are added or removed",
        ],
        docs: &[
            DocLink { label: "Consistent hashing", url: "https://en.wikipedia.org/wiki/Consistent_hashing" },
            DocLink { label: "BTreeMap", url: "https://doc.rust-lang.org/std/collections/struct.BTreeMap.html" },
        ],
        tests: &[
            TestHints {
                test_name: "empty_ring",
                hints: &["An empty BTreeMap has no entries. get_node should return None."],
            },
            TestHints {
                test_name: "single_node",
                hints: &[
                    "add_node: for i in 0..self.vnodes insert (mmh3_mix(fnv1a(name) ^ i as u64), name.to_string()) into self.ring.",
                    "get_node: h = fnv1a(key). Find ring.range(h..).next() for clockwise lookup. Wrap to ring.iter().next() if empty.",
                ],
            },
            TestHints {
                test_name: "node_count",
                hints: &["Track physical nodes separately (a HashSet<String>), or count ring.len() / self.vnodes."],
            },
            TestHints {
                test_name: "removing_a_node",
                hints: &["remove_node: for i in 0..self.vnodes compute the same hash and ring.remove(&hash)."],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "bloom-filter", display: "14 · bloom-filter",
        concepts: &[
            "Probabilistic membership: never false negative, may false positive",
            "k hash functions set/check k bits per item",
            "False positive rate: (1 - e^(-kn/m))^k",
            "Larger m (bit array) or smaller n reduces false positives",
        ],
        docs: &[
            DocLink { label: "Bloom filter", url: "https://en.wikipedia.org/wiki/Bloom_filter" },
            DocLink { label: "FNV-1a hash", url: "https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function" },
        ],
        tests: &[
            TestHints {
                test_name: "inserted_item",
                hints: &[
                    "insert: for seed in 0..self.k { self.bits[hash_with_seed(item, seed as u64, m)] = true; }",
                    "contains: for seed in 0..self.k { if !self.bits[hash_with_seed(item, seed as u64, m)] { return false; } } true",
                ],
            },
            TestHints {
                test_name: "estimated_fpr",
                hints: &[
                    "fpr = (1.0 - (-self.k as f64 * self.n_inserted as f64 / m as f64).exp()).powi(self.k as i32)",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "rate-limiter", display: "15 · rate-limiter",
        concepts: &[
            "Token bucket: capacity cap, refilled at rate tokens/sec",
            "Lazy refill: compute elapsed since last_refill on each try_acquire call",
            "Sliding window: keep a deque of timestamps, evict old ones",
            "Used by: AWS API Gateway, Nginx, Stripe, GitHub",
        ],
        docs: &[
            DocLink { label: "Token bucket algorithm", url: "https://en.wikipedia.org/wiki/Token_bucket" },
            DocLink { label: "std::time::Instant", url: "https://doc.rust-lang.org/std/time/struct.Instant.html" },
        ],
        tests: &[
            TestHints {
                test_name: "new_bucket_starts_full",
                hints: &[
                    "available_tokens: call refill() first, then return self.tokens.",
                    "refill: elapsed = self.last_refill.elapsed().as_secs_f64(). tokens += elapsed * rate. Cap at capacity. Update last_refill.",
                ],
            },
            TestHints {
                test_name: "acquiring_more_than",
                hints: &["try_acquire: refill(), if tokens >= cost { tokens -= cost; true } else { false }"],
            },
            TestHints {
                test_name: "tokens_do_not_exceed",
                hints: &["self.tokens = (self.tokens + added).min(self.capacity)"],
            },
            TestHints {
                test_name: "old_requests_expire",
                hints: &[
                    "SlidingWindowLimiter::try_acquire: first drain timestamps older than self.window.",
                    "while let Some(&front) = self.timestamps.front() { if front.elapsed() > self.window { self.timestamps.pop_front(); } else { break; } }",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "merkle-tree", display: "16 · merkle-tree",
        concepts: &[
            "Binary hash tree: parent = hash(left_child || right_child)",
            "Root summarises ALL data - changing any leaf changes the root",
            "Inclusion proofs: O(log n) - just the sibling path to root",
            "Used in: Bitcoin, git, certificate transparency, AWS S3",
        ],
        docs: &[
            DocLink { label: "Merkle tree", url: "https://en.wikipedia.org/wiki/Merkle_tree" },
            DocLink { label: "Bitcoin Merkle trees", url: "https://en.bitcoin.it/wiki/Protocol_documentation#Merkle_Trees" },
        ],
        tests: &[
            TestHints {
                test_name: "single_leaf_root",
                hints: &[
                    "build: hash each block with fnv1a(). levels[0] = leaf hashes. Then pair up and combine level by level.",
                    "If odd number of leaves, duplicate the last: level.push(*level.last().unwrap()).",
                ],
            },
            TestHints {
                test_name: "different_data_produces",
                hints: &["parent_hash = combine(left, right) where combine is already implemented for you."],
            },
            TestHints {
                test_name: "proof_verifies",
                hints: &[
                    "For leaf at index i: sibling index = i ^ 1. If i is even, sibling is on Right; if odd, on Left.",
                    "proof: walk up the levels, collecting (sibling_hash, side) pairs.",
                ],
            },
            TestHints {
                test_name: "tampered_data",
                hints: &[
                    "verify: hash data with fnv1a(). For each (sibling, side): if Left: combine(sibling, current); if Right: combine(current, sibling).",
                    "Compare final hash to root. Tampered data produces a different hash at the leaf, propagating up.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "gossip-protocol", display: "17 · gossip-protocol",
        concepts: &[
            "Epidemic protocol: informed nodes infect random neighbours each round",
            "Convergence in O(log n) rounds with fanout >= 2",
            "Eventual consistency - all nodes eventually have the same state",
            "Used in: Cassandra, Consul, blockchain peer discovery",
        ],
        docs: &[
            DocLink { label: "Gossip protocol", url: "https://en.wikipedia.org/wiki/Gossip_protocol" },
            DocLink { label: "Cassandra gossip", url: "https://cassandra.apache.org/doc/latest/cassandra/architecture/gossip.html" },
        ],
        tests: &[
            TestHints {
                test_name: "broadcast_marks_origin",
                hints: &[
                    "broadcast(origin, value): self.nodes[origin].value = Some(value); self.target_value = Some(value);",
                ],
            },
            TestHints {
                test_name: "gossip_reaches_all",
                hints: &[
                    "step: collect indices of all informed nodes (value == target). For each, pick fanout random OTHER nodes and set their value.",
                    "Increment self.rounds after each step.",
                ],
            },
            TestHints {
                test_name: "informed_count_grows",
                hints: &[
                    "informed_count = nodes where value == self.target_value. converged = informed_count == nodes.len().",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "raft-consensus", display: "18 · raft-consensus",
        concepts: &[
            "Raft: understandable consensus algorithm (simpler than Paxos)",
            "Term: logical clock that increases with each election",
            "Leader election: candidate collects majority votes",
            "Log replication: leader appends, followers accept, majority = committed",
        ],
        docs: &[
            DocLink { label: "Raft paper", url: "https://raft.github.io/raft.pdf" },
            DocLink { label: "Raft visualisation", url: "https://raft.github.io" },
        ],
        tests: &[
            TestHints {
                test_name: "cluster_of_one",
                hints: &[
                    "tick: node 0 becomes Candidate, increments term, votes for itself. With n=1, 1 vote = majority. Becomes Leader.",
                ],
            },
            TestHints {
                test_name: "three_node_cluster",
                hints: &[
                    "Node 0 requests votes from nodes still on old term. They grant it and update their term. 2/3 = majority -> Leader.",
                ],
            },
            TestHints {
                test_name: "committed_log",
                hints: &[
                    "append: leader creates LogEntry { term, data }, appends to leader.log, replicates to all follower logs.",
                    "committed_log: entries present in a majority of nodes' logs in the same order.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "gradient-descent", display: "19 · gradient-descent",
        concepts: &[
            "SGD: velocity = beta*v + (1-beta)*grad; params -= lr * velocity",
            "Adam: adaptive per-parameter learning rates using moment estimates",
            "Bias correction: m_hat = m / (1 - beta1^t) prevents cold-start decay",
            "Numerical gradient: central differences (f(x+h) - f(x-h)) / 2h",
        ],
        docs: &[
            DocLink { label: "Adam paper (Kingma & Ba 2014)", url: "https://arxiv.org/abs/1412.6980" },
            DocLink { label: "Gradient descent", url: "https://en.wikipedia.org/wiki/Gradient_descent" },
        ],
        tests: &[
            TestHints {
                test_name: "zero_lr",
                hints: &["If lr=0, velocity is updated but params -= 0*v = no change. params should stay the same."],
            },
            TestHints {
                test_name: "step_moves_in_negative",
                hints: &[
                    "SGD: if velocity is empty, initialise to zeros. velocity[i] = momentum*v[i] + (1-momentum)*grad[i].",
                    "params[i] -= lr * velocity[i]. Positive gradient -> decrease params.",
                ],
            },
            TestHints {
                test_name: "converges_to_minimum_of_x",
                hints: &[
                    "For f(x)=x², grad=2x. With lr=0.1 and no momentum, x converges to 0 in ~200 steps.",
                    "Adam: m = beta1*m + (1-beta1)*g; v = beta2*v + (1-beta2)*g*g; t++; m_hat=m/(1-beta1^t); v_hat=v/(1-beta2^t); params -= lr*m_hat/(sqrt(v_hat)+eps).",
                ],
            },
            TestHints {
                test_name: "numerical_gradient_of_x_squared",
                hints: &[
                    "numerical_gradient: for each dim i, perturb x±h on that axis only. grad[i] = (f(x+h*e_i) - f(x-h*e_i)) / (2*h).",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "neural-net", display: "20 · neural-net",
        concepts: &[
            "Forward pass: z = W*a + b, a = sigmoid(z) for each layer",
            "Backprop output delta: a_L - y (BCE + sigmoid simplifies beautifully)",
            "Hidden delta: (W_next)^T * delta_next * sigmoid'(z)",
            "Weight update: W -= lr * delta × a_prev^T",
        ],
        docs: &[
            DocLink { label: "Backpropagation", url: "https://en.wikipedia.org/wiki/Backpropagation" },
            DocLink { label: "Neural networks and deep learning (free book)", url: "http://neuralnetworksanddeeplearning.com" },
        ],
        tests: &[
            TestHints {
                test_name: "forward_output_shape",
                hints: &[
                    "Forward: start with a = input. For layer l: z[j] = sum_k(weights[l][j][k] * a[k]) + biases[l][j]. a = sigmoid(z).",
                ],
            },
            TestHints {
                test_name: "learns_xor",
                hints: &[
                    "train: for each epoch, iterate all (input, target) pairs and call backprop.",
                    "Backprop: forward pass storing all z and a. Output delta = a_L - y. For hidden: delta = W_next^T * delta_next elem-wise * sigmoid_deriv(z).",
                    "Update W[l][j][k] -= lr * delta[j] * a_prev[k]. Update bias[l][j] -= lr * delta[j].",
                ],
            },
            TestHints {
                test_name: "accuracy_improves",
                hints: &["accuracy: for each sample, output = forward(x). Predicted = (output[i] >= 0.5). Count matches / total."],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "decision-tree", display: "21 · decision-tree",
        concepts: &[
            "CART: Classification And Regression Trees",
            "Gini impurity: 1 - Σ p_i². Pure node=0, 50/50 split=0.5",
            "Information gain: Gini(parent) - weighted_avg(Gini(left), Gini(right))",
            "Stop conditions: all same class, max_depth reached, no gain",
        ],
        docs: &[
            DocLink { label: "Decision tree learning", url: "https://en.wikipedia.org/wiki/Decision_tree_learning" },
            DocLink { label: "Gini impurity", url: "https://en.wikipedia.org/wiki/Decision_tree_learning#Gini_impurity" },
        ],
        tests: &[
            TestHints {
                test_name: "gini_pure_node",
                hints: &["gini = 1 - Σ p_i². p = count / total. Pure node: p=1 -> gini = 0."],
            },
            TestHints {
                test_name: "gini_balanced",
                hints: &["50/50 split: p_true=p_false=0.5. gini = 1 - (0.25 + 0.25) = 0.5."],
            },
            TestHints {
                test_name: "information_gain_is_positive",
                hints: &[
                    "gain = gini(parent) - (|left|/|parent|)*gini(left) - (|right|/|parent|)*gini(right).",
                    "Perfect split: gini(left)=0, gini(right)=0 -> gain = gini(parent) > 0.",
                ],
            },
            TestHints {
                test_name: "linearly_separable",
                hints: &[
                    "build_node: if pure or depth==max_depth -> Leaf. Else best_split -> Split { left: build_node(...), right: build_node(...) }.",
                    "best_split: try each feature. Sort values, try thresholds = midpoints between adjacent values. Return (feature, threshold) with max gain.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "k-means", display: "22 · k-means",
        concepts: &[
            "Lloyd's algorithm: assign -> update centroids -> repeat",
            "Euclidean distance: sqrt(Σ (a_i - b_i)²)",
            "Initialise centroids by sampling k distinct data points",
            "Convergence: centroids stop moving (or max_iter reached)",
        ],
        docs: &[
            DocLink { label: "K-means clustering", url: "https://en.wikipedia.org/wiki/K-means_clustering" },
            DocLink { label: "Lloyd's algorithm", url: "https://en.wikipedia.org/wiki/Lloyd%27s_algorithm" },
        ],
        tests: &[
            TestHints {
                test_name: "identical_points_have_zero",
                hints: &["euclidean_distance = a.iter().zip(b).map(|(ai,bi)| (ai-bi).powi(2)).sum::<f64>().sqrt()"],
            },
            TestHints {
                test_name: "k1_assigns",
                hints: &[
                    "Initialise: pick k random (distinct) indices from data as starting centroids.",
                    "Assign step: predict(point) = index of closest centroid. Update: new centroid = mean of assigned points.",
                ],
            },
            TestHints {
                test_name: "three_well_separated",
                hints: &[
                    "Convergence check: if no centroid moved more than 1e-10, stop early.",
                    "If a cluster has no points after assignment, keep the old centroid (avoid NaN from empty mean).",
                ],
            },
            TestHints {
                test_name: "inertia",
                hints: &["inertia = Σ euclidean_distance(point, centroid[predict(point)])²"],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "attention-mechanism", display: "23 · attention-mechanism",
        concepts: &[
            "Attention(Q,K,V) = softmax(QK^T / sqrt(d_k)) V",
            "Scaling by 1/sqrt(d_k) prevents vanishing gradients in deep networks",
            "Causal mask: set future positions to -inf before softmax",
            "The computational core of GPT, BERT, LLaMA, Whisper, Claude",
        ],
        docs: &[
            DocLink { label: "Attention Is All You Need (2017)", url: "https://arxiv.org/abs/1706.03762" },
            DocLink { label: "The Illustrated Transformer", url: "http://jalammar.github.io/illustrated-transformer/" },
        ],
        tests: &[
            TestHints {
                test_name: "outputs_sum_to_one",
                hints: &[
                    "softmax(x): subtract max for stability. e_i = exp(x_i - max). result_i = e_i / sum(e).",
                ],
            },
            TestHints {
                test_name: "output_shape",
                hints: &[
                    "Step 1: scores = matmul_2d(q, &transpose_2d(k)). Divide each element by sqrt(d_k).",
                    "Step 2: apply softmax to each row of scores. Step 3: output = matmul_2d(&weights, v).",
                ],
            },
            TestHints {
                test_name: "identical_q_k",
                hints: &["When Q=K, each row of scores is identical -> softmax gives uniform weights -> output = mean of V rows."],
            },
            TestHints {
                test_name: "causal_mask",
                hints: &[
                    "Before softmax: for i in 0..seq_len { for j in (i+1)..seq_len { scores[i][j] = f64::NEG_INFINITY; } }",
                    "After masking: softmax of row 0 will give weight 1.0 to position 0 only. output[0] = v[0].",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "bpe-tokeniser", display: "24 · bpe-tokeniser",
        concepts: &[
            "BPE: start from characters, greedily merge the most frequent adjacent pair",
            "Each merge creates a new token and reduces the total sequence length",
            "Encoding: apply learned merge rules in order to new text",
            "Used in GPT-2/3/4, LLaMA, Whisper, Claude",
        ],
        docs: &[
            DocLink { label: "BPE paper (Sennrich et al. 2015)", url: "https://arxiv.org/abs/1508.07909" },
            DocLink { label: "tiktoken (OpenAI)", url: "https://github.com/openai/tiktoken" },
        ],
        tests: &[
            TestHints {
                test_name: "pair_frequencies",
                hints: &[
                    "For each sequence, slide window of 2: freqs[(&seq[i], &seq[i+1])] += 1.",
                    "Use HashMap<(String,String), usize>. Count all adjacent pairs across all sequences.",
                ],
            },
            TestHints {
                test_name: "merge_pair_replaces",
                hints: &[
                    "Scan tokens left to right. When tokens[i]==pair.0 && tokens[i+1]==pair.1, push merged token and skip i+1.",
                ],
            },
            TestHints {
                test_name: "most_frequent_pair",
                hints: &[
                    "train: split corpus into char sequences. Repeat: pair_frequencies -> find max pair -> merge_pair on all seqs -> add new token to vocab.",
                    "Stop when vocab_size reaches target. The first merge is the globally most frequent pair.",
                ],
            },
            TestHints {
                test_name: "encode_decode_roundtrip",
                hints: &[
                    "encode: start with each char as a token. For each merge rule in self.merges order, apply merge_pair.",
                    "decode: self.id_to_token[id] for each id. Concatenate the strings.",
                ],
            },
            TestHints {
                test_name: "trained_tokeniser_produces_fewer",
                hints: &[
                    "After training on 'aaaa', the pair ('a','a') is merged into 'aa'. 'aaaa' -> ['aa','aa'] -> 2 tokens instead of 4.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "mmio-registers", display: "25 · mmio-registers",
        concepts: &[
            "Memory-mapped I/O: hardware registers as fixed memory addresses",
            "volatile reads/writes: why the compiler must not cache hardware values",
            "Bitfield extraction and insertion (extends 09-bit-manipulator)",
            "HAL: hardware abstraction layer pattern (embedded-hal, embassy)",
        ],
        docs: &[
            DocLink { label: "core::ptr::read_volatile", url: "https://doc.rust-lang.org/core/ptr/fn.read_volatile.html" },
            DocLink { label: "embedded-hal traits", url: "https://docs.rs/embedded-hal/latest/embedded_hal/" },
        ],
        tests: &[
            TestHints {
                test_name: "write_then_read",
                hints: &[
                    "Use unsafe { std::ptr::read_volatile(self.buffer.as_ptr().add(index)) } to read.",
                    "Use unsafe { std::ptr::write_volatile(self.buffer.as_mut_ptr().add(index), value) } to write.",
                ],
            },
            TestHints {
                test_name: "out_of_bounds",
                hints: &[
                    "Check index < self.buffer.len() first. Return Err(RegError::OutOfBounds) if not.",
                ],
            },
            TestHints {
                test_name: "readonly_register",
                hints: &[
                    "Check self.perms[index] before writing. Permission::ReadOnly -> Err(RegError::PermissionDenied).",
                ],
            },
            TestHints {
                test_name: "read_field",
                hints: &[
                    "Call read32(index)?, then apply: (value >> bit_offset) & ((1u64 << len) - 1) as u32.",
                    "Use u64 for the mask to avoid overflow when len == 32: ((1u64 << len) - 1) as u32.",
                ],
            },
            TestHints {
                test_name: "write_field",
                hints: &[
                    "Build mask: ((1u64 << len) - 1) as u32) << bit_offset. Read current, clear field, OR new bits.",
                    "let current = read32(index)?; let cleared = current & !mask; write32(index, cleared | ((value << bit_offset) & mask))",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "char-device-driver", display: "26 · char-device-driver",
        concepts: &[
            "file_operations table (fops): drivers are just function pointers",
            "Character devices: open/read/write/ioctl as the universal hardware API",
            "Exclusive access: why hardware resources need locking (EBUSY)",
            "ioctl: structured escape hatch for device-specific commands",
        ],
        docs: &[
            DocLink { label: "Linux char device driver guide", url: "https://www.kernel.org/doc/html/latest/driver-api/basics.html" },
            DocLink { label: "POSIX ioctl(2)", url: "https://man7.org/linux/man-pages/man2/ioctl.2.html" },
        ],
        tests: &[
            TestHints {
                test_name: "open_succeeds",
                hints: &[
                    "SimpleDevice::new() should return Self { is_open: false, buffer: vec![], mode: 0 }.",
                    "open(): if self.is_open { Err(Busy) } else { self.is_open = true; Ok(()) }",
                ],
            },
            TestHints {
                test_name: "second_open_returns_busy",
                hints: &[
                    "Check self.is_open at the top of open(). Return Err(DeviceError::Busy) if true.",
                ],
            },
            TestHints {
                test_name: "write_then_read",
                hints: &[
                    "write(): check is_open, then self.buffer.extend_from_slice(buf). Return Ok(buf.len()).",
                    "read(): check is_open, let n = buf.len().min(self.buffer.len()); buf[..n].copy_from_slice(&self.buffer[..n]); self.buffer.drain(..n); Ok(n)",
                ],
            },
            TestHints {
                test_name: "reset_clears",
                hints: &[
                    "ioctl cmd::RESET: self.buffer.clear(); Ok(0)",
                ],
            },
            TestHints {
                test_name: "unknown_command",
                hints: &[
                    "The match on cmd needs a wildcard arm: _ => Err(DeviceError::InvalidCommand)",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "process-scheduler", display: "27 · process-scheduler",
        concepts: &[
            "MLFQ: multi-level feedback queue approximates optimal without clairvoyance",
            "Time quantum and preemption: why slices prevent CPU monopolisation",
            "I/O vs CPU-bound: how usage patterns drive priority demotion / promotion",
            "Starvation and ageing: boosting long-waiting processes for fairness",
        ],
        docs: &[
            DocLink { label: "OSTEP: MLFQ (ch. 8)", url: "https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched-mlfq.pdf" },
            DocLink { label: "Linux EEVDF scheduler", url: "https://lwn.net/Articles/925371/" },
        ],
        tests: &[
            TestHints {
                test_name: "empty_scheduler",
                hints: &[
                    "new(): queues = std::array::from_fn(|_| VecDeque::new()), processes: HashMap::new(), tick: 0, next_pid: 1.",
                    "next_process(): for queue in &mut self.queues { if let Some(pid) = queue.pop_front() { return Some(pid); } } None",
                ],
            },
            TestHints {
                test_name: "higher_priority",
                hints: &[
                    "spawn(): create Pcb { pid, name, state: Ready, queue: 0, ... }. Insert into processes map. Push pid to queues[0].",
                ],
            },
            TestHints {
                test_name: "cpu_bound_process_demotes",
                hints: &[
                    "In tick(): pcb.ticks_this_quantum += 1; pcb.total_cpu_ticks += 1. If ticks_this_quantum >= QUANTA[queue]: pcb.queue = (queue + 1).min(NUM_QUEUES - 1); pcb.ticks_this_quantum = 0.",
                    "After updating queue, push pid back onto queues[pcb.queue].",
                ],
            },
            TestHints {
                test_name: "io_bound_process_is_promoted",
                hints: &[
                    "If yielded_early: pcb.queue = queue.saturating_sub(1); pcb.ticks_this_quantum = 0. Push back to new queue.",
                ],
            },
            TestHints {
                test_name: "starved_process_is_aged",
                hints: &[
                    "age_processes(): for each pid in processes where state==Ready and wait_ticks >= AGE_THRESHOLD: remove from current queue, set queue=0, reset wait_ticks, push to queues[0].",
                    "Call age_processes() in tick() when self.tick % AGE_THRESHOLD == 0.",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "raw-socket", display: "28 · raw-socket",
        concepts: &[
            "Wire format: reading RFC-defined byte layouts without a library",
            "Ethernet / IPv4 / TCP header structures and field offsets",
            "IPv4 one's-complement checksum validation",
            "Connection tracking: identifying repeated flows by 4-tuple",
        ],
        docs: &[
            DocLink { label: "RFC 791 (IPv4)", url: "https://www.rfc-editor.org/rfc/rfc791" },
            DocLink { label: "RFC 793 (TCP)", url: "https://www.rfc-editor.org/rfc/rfc793" },
            DocLink { label: "XDP tutorial", url: "https://github.com/xdp-project/xdp-tutorial" },
        ],
        tests: &[
            TestHints {
                test_name: "ethertype_is_parsed",
                hints: &[
                    "Ethernet: bytes 0-5 = dst MAC, 6-11 = src MAC, 12-13 = EtherType (big-endian u16).",
                    "let ethertype = u16::from_be_bytes([buf[12], buf[13]]); check for ETHERTYPE_IPV4 or ETHERTYPE_ARP.",
                ],
            },
            TestHints {
                test_name: "ipv4_checksum_validates",
                hints: &[
                    "Sum all 16-bit big-endian words in the header as u32. Fold carry: while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16) }",
                    "One's complement: let result = !sum as u16. A valid header gives 0xFFFF.",
                ],
            },
            TestHints {
                test_name: "tcp_flags",
                hints: &[
                    "TCP flags are in byte 13 of the TCP header (0-indexed from TCP payload start).",
                    "SYN = bit 1 (0x02), ACK = bit 4 (0x10), FIN = bit 0 (0x01), RST = bit 2 (0x04).",
                ],
            },
            TestHints {
                test_name: "connection_tracking",
                hints: &[
                    "Encode 4-tuple: (u32::from_be_bytes(ip.src), u32::from_be_bytes(ip.dst), tcp.src_port, tcp.dst_port)",
                    "!self.seen.insert(key) returns true if already present (HashSet::insert returns false on duplicate).",
                ],
            },
        ],
    },
    // ------------------------------------------------------------------
    CrateMeta {
        package: "ebpf-probe", display: "29 · ebpf-probe",
        concepts: &[
            "BPF bytecode and the in-kernel verifier: safety without a language runtime",
            "BPF maps: shared key-value memory between kernel programs and userspace",
            "kprobes and tracepoints: attaching to kernel function entry/exit points",
            "XDP: processing packets before the kernel network stack (line-rate filtering)",
            "CO-RE (Compile Once Run Everywhere) and BTF type information",
        ],
        docs: &[
            DocLink { label: "aya book", url: "https://aya-rs.dev/book/" },
            DocLink { label: "Brendan Gregg BPF Performance Tools", url: "https://www.brendangregg.com/bpf-performance-tools-book.html" },
            DocLink { label: "XDP tutorial", url: "https://github.com/xdp-project/xdp-tutorial" },
        ],
        tests: &[
            TestHints {
                test_name: "load_execve_probe",
                hints: &[
                    "let elf = include_bytes!(\"../bpf/execve_probe.bpf.o\"); let mut bpf = aya::Bpf::load(elf)?;",
                    "let prog: &mut KProbe = bpf.program_mut(\"execve_probe\").unwrap().try_into()?; prog.load()?; prog.attach(\"sys_execve\", 0)?;",
                ],
            },
            TestHints {
                test_name: "read_exec_events",
                hints: &[
                    "let map = aya::maps::HashMap::<_, u32, [u8; 16]>::try_from(bpf.map(\"exec_events\").unwrap())?;",
                    "for result in map.iter() { let (pid, comm) = result?; let name = String::from_utf8_lossy(&comm).trim_end_matches('\\0').to_string(); }",
                ],
            },
            TestHints {
                test_name: "protocol_classifier",
                hints: &[
                    "Unit tests in the tests module are already passing - they test the inline classify_protocol helper.",
                    "For the real eBPF functions, you need Linux >= 5.15 with BTF. Check /sys/kernel/btf/vmlinux exists.",
                ],
            },
        ],
    },
];
