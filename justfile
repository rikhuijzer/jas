default:
    just --list

# Run the Rust harness against the standalone Bash script.
test:
    cargo test --locked

# Check Bash and test-harness formatting.
check:
    bash -n jas
    shellcheck jas
    cargo fmt --all -- --check
    cargo clippy --tests --locked -- -D warnings

# Create and push an annotated release tag after confirmation.
tag:
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ "$(git branch --show-current)" != "main" ]]; then
        echo "Release from the main branch." >&2
        exit 1
    fi
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Commit your changes before releasing." >&2
        exit 1
    fi

    version=$(bash ./jas --version)
    version=${version#jas }
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
        echo "Invalid jas version: $version" >&2
        exit 1
    fi
    tag="v$version"

    git fetch origin main
    if [[ "$(git rev-parse HEAD)" != "$(git rev-parse FETCH_HEAD)" ]]; then
        echo "Push main and ensure it matches origin/main before releasing." >&2
        exit 1
    fi
    if git show-ref --verify --quiet "refs/tags/$tag"; then
        echo "Tag $tag already exists locally." >&2
        exit 1
    fi
    remote_tag=$(git ls-remote --tags origin "refs/tags/$tag")
    if [[ -n "$remote_tag" ]]; then
        echo "Tag $tag already exists on origin." >&2
        exit 1
    fi

    remote=$(git config --get remote.origin.url)
    case "$remote" in
        https://github.com/*) ;;
        git@github.com:*) remote="https://github.com/${remote#git@github.com:}" ;;
        ssh://git@github.com/*) remote="https://github.com/${remote#ssh://git@github.com/}" ;;
        *) echo "Expected a GitHub origin remote." >&2; exit 1 ;;
    esac
    remote=${remote%.git}
    notes="See [CHANGELOG.md]($remote/blob/$tag/CHANGELOG.md) for more information about changes since the last release."

    printf 'Create and push %s at %s? This triggers the GitHub release workflow.\n\n' "$tag" "$(git rev-parse --short HEAD)"
    printf '%s\n\n' "$notes"
    read -r -p "Type YES to continue: " reply
    if [[ "$reply" != "YES" ]]; then
        echo "Release cancelled."
        exit 1
    fi

    git tag -a "$tag" -m "$notes"
    if ! git push origin "refs/tags/$tag"; then
        echo "Push failed; local tag $tag was kept. Retry with: git push origin refs/tags/$tag" >&2
        exit 1
    fi
