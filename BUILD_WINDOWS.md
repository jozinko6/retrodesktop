# Windows build

## Toolchain

1. Install Rust stable with `x86_64-pc-windows-msvc`.
2. Install Visual Studio 2022 Build Tools and Desktop C++ workload.
3. Install WebView2 Runtime, Node.js 22+, and pnpm 10+.

## Verify

```powershell
node --version
pnpm --version
rustc --version
cargo --version
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

Expected installers:

- `src-tauri/target/release/bundle/nsis/*.exe`
- `src-tauri/target/release/bundle/msi/*.msi`

The Windows build was verified on June 23, 2026 and produced both bundles.

A portable ZIP can be assembled from the release executable and required Tauri runtime files after a successful signed build. It must not include user data or managed emulators.
