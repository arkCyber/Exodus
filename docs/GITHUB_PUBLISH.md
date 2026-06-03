# Publishing Exodus to GitHub

Checklist before the first push.

## 1. Secrets and large files

- [ ] No `.env` in the commit (only `.env.example`)
- [ ] `allama/models/` is **not** tracked (8GB+ weights; listed in `.gitignore`)
- [ ] `**/target/` and `node_modules/` are not tracked
- [ ] No API keys in source or docs

Verify:

```bash
git status
git check-ignore -v allama/models/gemma-4-E4B 2>/dev/null || true
```

## 2. First commit (local)

```bash
cd /path/to/Exodus
git add -A
git status   # review list carefully
git commit -m "$(cat <<'EOF'
Initial commit: Exodus Browser (Tauri + SvelteKit + Rust).

Privacy-first AI browser with local sidecar, lifecycle monitoring,
microservices, and optional IM stack (tests gated behind im-tests).
EOF
)"
```

## 3. Create GitHub repository

**Option A — GitHub CLI**

```bash
gh auth login
gh repo create Exodus --public --source=. --remote=origin --push
```

**Option B — Manual**

1. Create an empty repo on GitHub (no README/license if you already have them locally).
2. `git remote add origin git@github.com:YOUR_USER/Exodus.git`
3. `git branch -M main`
4. `git push -u origin main`

## 4. After push

- [ ] Enable GitHub Actions (CI workflow in `.github/workflows/ci.yml`)
- [ ] Add repository description and topics: `browser`, `tauri`, `rust`, `privacy`, `local-ai`
- [ ] Set default branch to `main`
- [ ] Add maintainer contact in `SECURITY.md`
- [ ] Optional: rename `master` → `main` if still on `master`

## 5. CI notes

- CI runs `scripts/verify.sh` on Ubuntu (includes Tauri system deps).
- Local Rust tests skip IM by default: `./scripts/cargo-test-non-im.sh`
- IM tests: `cargo test -p exodus-tauri --features im-tests`

## 6. Optional cleanup (recommended later)

Many audit/report `.md` files live at repo root; consider moving to `docs/reports/` in a follow-up PR for a cleaner GitHub landing page.
