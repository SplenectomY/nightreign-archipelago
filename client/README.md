# Client (`nightreign_ap.dll`)

Phase 0: load into `nightreign.exe`, open a console, read `flags.toml`.

```
cd client
cargo build --release
```

Needs the MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`).

Copy `target/release/nightreign_ap.dll` next to a `flags.toml` (copy `data/flags.toml`) so the worker can find it.

Archipelago WebSocket connect is intentionally not in this crate yet. Attach has to work first.
