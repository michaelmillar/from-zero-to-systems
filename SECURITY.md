# Security Policy

## Supported Versions

This is an educational Rust workspace. There are no production deployments or versioned releases.

## Reporting a Vulnerability

If you discover a security issue in this repository (e.g. a dependency with a known CVE, or unsafe code that is unsound), please open a GitHub issue with the label `security`.

There is no SLA or bounty programme - this is a learning project.

## Unsafe Code

Some crates (`10-memory-arena`) intentionally use `unsafe` Rust as a teaching exercise. All `unsafe` blocks carry a `SAFETY:` comment explaining the invariants upheld.
