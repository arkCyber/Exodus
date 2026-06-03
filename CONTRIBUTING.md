# Contributing to Exodus

Thank you for your interest in Exodus Browser.

## Development setup

1. Install [Node.js](https://nodejs.org/) 22+, [pnpm](https://pnpm.io/) 9+, and [Rust](https://rust.rust-lang.org/) stable.
2. Clone the repository and install dependencies:

   ```bash
   pnpm install
   ```

3. Run the app (Tauri + Vite):

   ```bash
   pnpm tauri dev
   ```

4. Run checks before opening a PR:

   ```bash
   pnpm verify          # full suite (see scripts/verify.sh)
   pnpm test:quick      # faster loop
   pnpm test:rust       # Rust only, skips IM tests by default
   ```

## IM / messaging tests

Instant-messaging modules are gated behind the `im-tests` Cargo feature until the network stack is configured:

```bash
./scripts/cargo-test-im.sh
```

## Pull requests

- Target branch: `main` (or `master` until renamed).
- Keep PRs focused; link related issues when applicable.
- Ensure CI passes (`/.github/workflows/ci.yml`).
- Do not commit secrets, `.env`, or model weights under `allama/models/`.

## Code style

- Rust: `cargo fmt` / `cargo clippy` where applicable.
- Frontend: follow existing Svelte/TypeScript patterns; run `pnpm check`.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.
