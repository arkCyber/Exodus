#!/usr/bin/env bash
# Install a git pre-push hook that runs ./scripts/verify.sh before every push.
# Usage: ./scripts/install-git-hooks.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK_DIR="$ROOT/.git/hooks"
HOOK_FILE="$HOOK_DIR/pre-push"

if [[ ! -d "$ROOT/.git" ]]; then
  echo "Error: not a git repository ($ROOT)" >&2
  exit 1
fi

mkdir -p "$HOOK_DIR"

cat > "$HOOK_FILE" << 'EOF'
#!/usr/bin/env bash
# Exodus pre-push — run full verify suite (installed by scripts/install-git-hooks.sh)
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
exec "$ROOT/scripts/verify.sh"
EOF

chmod +x "$HOOK_FILE"
chmod +x "$ROOT/scripts/verify.sh"
chmod +x "$ROOT/scripts/test-quick.sh"

echo "Installed pre-push hook → scripts/verify.sh"
echo "To skip once: git push --no-verify"
