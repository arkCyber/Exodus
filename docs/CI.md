# CI and automated testing

## GitHub Actions

| Workflow | When | What runs |
|----------|------|-------------|
| [**CI**](../.github/workflows/ci.yml) | Every PR; push to `main`/`master` | **frontend-auto**: `pnpm check` + `pnpm test:auto:ci` (full Vitest + Playwright) |
| [**CI**](../.github/workflows/ci.yml) `verify` job | Push to `main`/`master` only | `scripts/verify.sh` + coverage (Rust + integration) |
| [**Verify Nightly**](../.github/workflows/verify-nightly.yml) | Daily 06:00 UTC; manual | Full frontend gate + `verify.sh` + coverage |

### PRs (fast, ~3–5 min)

- Typecheck (`vue-tsc`)
- Full Vitest (`pnpm test`, all `src/**/*.test.ts`)
- Playwright `e2e/vue-shell-qa.spec.ts` (Vite, no Tauri)

Fast subset only: `pnpm test:auto:e2e`. Full Vitest also runs in `verify.sh` on `main` push and nightly.

**No** full Rust verify on PRs (keeps feedback fast).

### `main` branch push

Same as PR, then **verify** job (Rust `exodus-core`, `exodus-tauri`, integration scripts).

### Manual full verify on a branch

Actions → **CI** → **Run workflow** → enable **full_verify**.

### Nightly

[**Verify Nightly**](../.github/workflows/verify-nightly.yml) runs the full stack once per day even without merges.

## Local commands

```bash
pnpm test:auto          # unit only
pnpm test:auto:e2e      # unit + Playwright (spawns Vite)
pnpm test:auto:full     # + verify-quick

# Tauri backend E2E (two terminals):
pnpm tauri dev
pnpm test:e2e:tauri
```

Manual checklist: [`MANUAL_QA_VUE_SHELL.md`](./MANUAL_QA_VUE_SHELL.md).
