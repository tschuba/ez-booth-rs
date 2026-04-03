#!/bin/bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    cat <<'EOF'
Usage: ./scripts/create-release.sh [version]

Creates a release commit and annotated tag from an up-to-date local main branch.

Examples:
  ./scripts/create-release.sh
  ./scripts/create-release.sh 0.1.0
EOF
}

cleanup() {
    if [[ -n "${NOTES_FILE:-}" && -f "$NOTES_FILE" ]]; then
        rm -f "$NOTES_FILE"
    fi
}

require_clean_worktree() {
    if [[ -n "$(git status --short)" ]]; then
        echo "Working tree must be clean before creating a release." >&2
        exit 1
    fi
}

require_branch_main() {
    local current_branch
    current_branch="$(git branch --show-current)"

    if [[ "$current_branch" != "main" ]]; then
        echo "Release creation must start from the local main branch." >&2
        exit 1
    fi
}

require_synced_main() {
    git fetch origin

    local counts behind ahead
    counts="$(git rev-list --left-right --count main...origin/main)"
    behind="${counts%%$'\t'*}"
    ahead="${counts##*$'\t'}"

    if [[ "$behind" != "0" || "$ahead" != "0" ]]; then
        echo "Local main must match origin/main before creating a release." >&2
        echo "Run: git pull --ff-only" >&2
        exit 1
    fi
}

prompt_version() {
    local input_version="${1:-}"

    if [[ -z "$input_version" ]]; then
        read -r -p "Release version (for tag vX.Y.Z): " input_version
    fi

    if [[ ! "$input_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Version must use semantic versioning like 0.1.0." >&2
        exit 1
    fi

    printf '%s\n' "$input_version"
}

ensure_tag_absent() {
    local tag_name="$1"

    if git rev-parse "$tag_name" >/dev/null 2>&1; then
        echo "Tag $tag_name already exists locally." >&2
        exit 1
    fi

    if git ls-remote --tags origin "refs/tags/$tag_name" | grep -q .; then
        echo "Tag $tag_name already exists on origin." >&2
        exit 1
    fi
}

update_workspace_version() {
    local version="$1"

    VERSION="$version" python3 - <<'PY'
from pathlib import Path
import os
import re

path = Path("Cargo.toml")
content = path.read_text()
updated, count = re.subn(
    r'(?m)^version = "[^"]+"$',
    f'version = "{os.environ["VERSION"]}"',
    content,
    count=1,
)
if count != 1:
    raise SystemExit("Failed to update workspace version in Cargo.toml")
path.write_text(updated)
PY
}

capture_notes() {
    local notes_file="$1"

    cat <<'EOF'
Enter optional release notes for the annotated tag.
These notes are prepended to the auto-generated GitHub release notes.
Press Ctrl-D on a new line to finish, or just press Ctrl-D immediately to skip.
EOF

    cat > "$notes_file" || true
}

main() {
    if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
        usage
        exit 0
    fi

    cd "$ROOT_DIR"

    require_clean_worktree
    require_branch_main
    require_synced_main

    local version tag_name commit_message
    version="$(prompt_version "${1:-}")"
    tag_name="v${version}"
    NOTES_FILE="$(mktemp)"
    commit_message="chore: bump version to ${version}"

    trap cleanup EXIT

    ensure_tag_absent "$tag_name"
    capture_notes "$NOTES_FILE"

    update_workspace_version "$version"

    if git diff --quiet -- Cargo.toml; then
        echo "Cargo.toml already uses version ${version}; no version bump commit needed."
    else
        git add Cargo.toml
        git commit -m "$commit_message"
    fi

    if [[ -s "$NOTES_FILE" ]]; then
        git tag -a "$tag_name" -F "$NOTES_FILE"
    else
        git tag -a "$tag_name" -m "Release ${tag_name}"
    fi

    git push origin main
    git push origin "$tag_name"

    echo "Created release tag $tag_name and pushed main + tag to origin."
    echo "Monitor the workflow at: https://github.com/tschuba/ez-booth-rs/actions/workflows/release.yml"
}

main "$@"
