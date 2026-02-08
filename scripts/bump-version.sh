#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION="${1#v}"

# Update workspace.package.version
sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" Cargo.toml

# Update workspace.dependencies versions for internal crates
sed -i '' "s/\(pacs-core = { path = \"pacs-core\", version = \"\)[^\"]*/\1$VERSION/" Cargo.toml
sed -i '' "s/\(pacs-cli = { path = \"pacs-cli\", version = \"\)[^\"]*/\1$VERSION/" Cargo.toml
sed -i '' "s/\(pacs-cli = { path = \"pacs-tui\", version = \"\)[^\"]*/\1$VERSION/" Cargo.toml

echo "✅ Bumped version to $VERSION"
cargo check --quiet
echo "✅ cargo check passed"
