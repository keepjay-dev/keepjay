#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Install the Keepjay CLI (docs/guide/getting-started.md):
#
#   curl -fsSL https://keepjay.com/install.sh | sh
#
# keepjay.com/install.sh redirects to this file on the main branch.
#
# Downloads the release archive for this machine from GitHub, checks it against the
# release's SHA256SUMS.txt, and puts `keepjay` in ~/.local/bin. It needs no root and changes
# nothing else: no shell profiles, no PATH edits, no telemetry.
#
#   KEEPJAY_VERSION       a release to install, e.g. 0.2.0-alpha.3 (default: the newest)
#   KEEPJAY_INSTALL_DIR   where to put `keepjay` (default: ~/.local/bin)
#   GITHUB_TOKEN          optional, used only for the GitHub API lookup of the newest release
#
# macOS (Apple silicon, Intel) and Linux (x86-64). On Windows, download the .zip from the
# releases page.

set -eu

REPO="keepjay-dev/keepjay"

say() { printf '%s\n' "$*"; }
err() {
    printf 'install.sh: %s\n' "$*" >&2
    exit 1
}
has() { command -v "$1" > /dev/null 2>&1; }

# fetch URL FILE: download URL to FILE (or stdout when FILE is -).
fetch() {
    if has curl; then
        if [ -n "${GITHUB_TOKEN:-}" ] && [ "${3:-}" = api ]; then
            curl -fsSL --retry 3 -H "Authorization: Bearer $GITHUB_TOKEN" -o "$2" "$1"
        else
            curl -fsSL --retry 3 -o "$2" "$1"
        fi
    elif has wget; then
        if [ -n "${GITHUB_TOKEN:-}" ] && [ "${3:-}" = api ]; then
            wget -q --header "Authorization: Bearer $GITHUB_TOKEN" -O "$2" "$1"
        else
            wget -q -O "$2" "$1"
        fi
    else
        err "needs curl or wget"
    fi
}

target() {
    os=$(uname -s)
    arch=$(uname -m)
    case "$os" in
        Darwin)
            # A shell running under Rosetta reports x86_64 on Apple silicon.
            if [ "$arch" = x86_64 ] && [ "$(sysctl -n sysctl.proc_translated 2> /dev/null || echo 0)" = 1 ]; then
                arch=arm64
            fi
            case "$arch" in
                arm64 | aarch64) echo aarch64-apple-darwin ;;
                x86_64) echo x86_64-apple-darwin ;;
                *) err "no build for macOS on $arch" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64 | amd64) echo x86_64-unknown-linux-musl ;;
                *) err "no build for Linux on $arch yet; build from source (see CONTRIBUTING.md)" ;;
            esac
            ;;
        MINGW* | MSYS* | CYGWIN*) err "on Windows, download the .zip from https://github.com/$REPO/releases" ;;
        *) err "no build for $os" ;;
    esac
}

latest_version() {
    # Every release so far is a pre-release, which /releases/latest skips, so take the
    # newest from the list.
    fetch "https://api.github.com/repos/$REPO/releases?per_page=1" - api |
        sed -n 's/.*"tag_name": *"v\([^"]*\)".*/\1/p' | head -n 1
}

sha256() {
    if has sha256sum; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif has shasum; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        err "needs sha256sum or shasum to check the download"
    fi
}

main() {
    target=$(target)
    version=${KEEPJAY_VERSION:-}
    version=${version#v}
    if [ -z "$version" ]; then
        version=$(latest_version)
        [ -n "$version" ] || err "couldn't find the newest release (set KEEPJAY_VERSION to pick one)"
    fi
    dir=${KEEPJAY_INSTALL_DIR:-"$HOME/.local/bin"}
    name="keepjay-$version-$target"
    base="https://github.com/$REPO/releases/download/v$version"

    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT INT TERM

    say "Downloading Keepjay $version for $target"
    fetch "$base/$name.tar.gz" "$tmp/$name.tar.gz" || err "download failed: $base/$name.tar.gz"
    fetch "$base/SHA256SUMS.txt" "$tmp/SHA256SUMS.txt" || err "download failed: $base/SHA256SUMS.txt"

    want=$(awk -v f="$name.tar.gz" '$2 == f { print $1 }' "$tmp/SHA256SUMS.txt")
    [ -n "$want" ] || err "$name.tar.gz isn't listed in SHA256SUMS.txt"
    got=$(sha256 "$tmp/$name.tar.gz")
    [ "$got" = "$want" ] || err "checksum mismatch for $name.tar.gz: expected $want, got $got"

    tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
    mkdir -p "$dir"
    # Copy, then rename into place, so a running keepjay is never left half-written.
    cp "$tmp/$name/keepjay" "$dir/.keepjay.new"
    chmod 755 "$dir/.keepjay.new"
    mv -f "$dir/.keepjay.new" "$dir/keepjay"

    say "Installed $("$dir/keepjay" --version) to $dir/keepjay"
    case ":$PATH:" in
        *":$dir:"*) ;;
        *)
            say ""
            say "$dir isn't on your PATH. Add this line to your shell's startup file"
            say "(~/.zshrc, ~/.bashrc or ~/.config/fish/config.fish), then open a new terminal:"
            case "${SHELL:-}" in
                */fish) say "  fish_add_path $dir" ;;
                *) say "  export PATH=\"$dir:\$PATH\"" ;;
            esac
            ;;
    esac
    say ""
    say "Next: keepjay alias install   (adds the short command kj)"
    say "      then see https://github.com/$REPO/blob/main/docs/guide/getting-started.md"
}

# Everything runs from here, so a download cut off halfway through runs nothing.
main "$@"
