# challenge-host

Minimal runnable host workspace for the shared adapter boundary.

Current slice:
- spawns a game adapter over stdio
- speaks the live `hazptr-adapter` JSON protocol
- loads handshake, challenge list, and a selected workspace
- serves a local three-pane web UI
- exposes `inspect` and `web` CLI commands

Example:

```bash
cargo run --manifest-path challenge-host/Cargo.toml -p host-cli -- \
  inspect \
  --adapter cargo \
  --adapter-cwd /home/markw/projects/studying/hazptr \
  --adapter-arg run \
  --adapter-arg --quiet \
  --adapter-arg --bin \
  --adapter-arg hazptr-adapter \
  --adapter-arg --
```

Print the local web URL:

```bash
cargo run --manifest-path challenge-host/Cargo.toml -p host-cli -- \
  web \
  --adapter /bin/bash \
  --adapter-cwd /home/markw/projects/studying/hazptr \
  --adapter-arg -lc \
  --adapter-arg 'CARGO_TARGET_DIR=/tmp/hazptr-adapter-target cargo run --quiet --bin hazptr-adapter -- --repo .' \
  --print-only
```

Each adapter now supplies its own favicon/title/subtitle, so the pinned tab for `hazptr` shows the hazptr logo and DSA blurb while `fzts` gets its logo and the marketing copy about building complex Rust applications.
