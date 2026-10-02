# jas

Just an installer.

This tool is meant to be used in situations where you want to install a script or binary in a reliable way, that is, you want to specify the SHA-256 checksum so that you can be sure that you are executing the thing you expected.
I wrote this tool in response to yet another GitHub Action supply chain attack.
See the [Background](#background) section for more details.

## Installation

Download `jas` from a release (or copy it from this repository), then:

```bash
mkdir -p "$HOME/.local/bin"
cp jas "$HOME/.local/bin/jas"
chmod +x "$HOME/.local/bin/jas"
export PATH="$HOME/.local/bin:$HOME/.jas/bin:$PATH"
jas --help
```

The installer is one standalone Bash file; it does not require Rust, Python, or
Node.js. It supports Bash 3.2 or newer, including macOS's system Bash and Git
Bash on Windows. On Windows, run it in Git Bash or with `bash jas ...`.

Dependencies are `curl`, `tar`, ordinary Unix utilities, and either `sha256sum`
or `shasum`. GitHub release lookup (`--gh`) also requires `jq`; ZIP extraction
requires `unzip` or a ZIP-capable `tar` (bsdtar). For `.tar.xz`, use an
xz-capable `tar`; GNU tar also requires `xz`.
These are standard tools on GitHub-hosted Linux, macOS, and Windows runners.
Self-hosted and minimal container runners must provide these dependencies and
Bash themselves.

Keep a reviewed copy of `jas` in your repository, or verify a downloaded copy
against a trusted checksum before running it. Checking an installed program's
checksum does not verify the installer itself.

## Usage

To install Typos from GitHub into `~/.jas/bin`, you can use:

```bash
jas install \
--gh crate-ci/typos@v1.31.1 \
--sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
```

This command uses the SHA for the Linux x86_64 release.
To get the SHA for other platforms, you can use `sha --url`.
For example,

```bash
jas sha \
--url github.com/crate-ci/typos/releases/download/v1.31.1/typos-v1.31.1-x86_64-unknown-linux-musl.tar.gz
```

## Usage in GitHub Actions

Install jas from the [v0.4.0 release](https://github.com/rikhuijzer/jas/releases/tag/v0.4.0)
using one of the steps below. Use `shell: bash` for installation and subsequent
jas commands.
The download URLs below will be available once v0.4.0 is published.

### Linux runners

For GitHub-hosted Linux runners, use `sha256sum` directly:

```yaml
- name: Install jas
  run: |
    mkdir -p "$RUNNER_TEMP/jas"
    cd "$RUNNER_TEMP/jas"
    curl --fail --location --retry 2 \
      https://github.com/rikhuijzer/jas/releases/download/v0.4.0/jas -o jas
    sha="437b2988dbc58d5867b908bb67366b766b9bcd3ed4b266a129f8d96d75301f6e"
    printf '%s  jas\n' "$sha" | sha256sum -c -
    chmod +x jas
    printf '%s\n' "$RUNNER_TEMP/jas" >> "$GITHUB_PATH"
```

For a macOS-only workflow, use the same example with `shasum -a 256 -c -`
in place of `sha256sum -c -`.

### Linux, macOS, and Windows runners

For a platform independent workflow:

```yaml
- name: Install jas
  shell: bash
  run: |
    mkdir -p "$RUNNER_TEMP/jas"
    cd "$RUNNER_TEMP/jas"
    curl --fail --location --retry 2 \
      https://github.com/rikhuijzer/jas/releases/download/v0.4.0/jas -o jas
    sha="437b2988dbc58d5867b908bb67366b766b9bcd3ed4b266a129f8d96d75301f6e"
    hash=(sha256sum)
    command -v sha256sum >/dev/null 2>&1 || hash=(shasum -a 256)
    printf '%s  jas\n' "$sha" | "${hash[@]}" -c -
    chmod +x jas
    printf '%s\n' "$RUNNER_TEMP/jas" >> "$GITHUB_PATH"
```

The hardcoded SHA-256 in both examples pins the v0.4.0 `jas` release asset. Update the version
and its reviewed checksum together when upgrading. Adding to `GITHUB_PATH`
makes jas available in subsequent steps. `RUNNER_TEMP` already uses the runner's
native path format, including on Windows.

After installing jas with either step above, to install and run [`typos`](https://github.com/crate-ci/typos) v1.31.1, add these steps to an Ubuntu x64 job:

```yaml
- uses: actions/checkout@v4
- name: Install typos
  shell: bash
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: >
    jas install
    --gh crate-ci/typos@v1.31.1
    --sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
- run: typos .
```

As stated above, the benefit of this is that you can be sure which version of the binary you are using.
If someone changes the binary, the SHA will change and your CI will fail.

The `--gh-token` is optional but recommended inside GitHub Actions because otherwise this tool might be rate limited when determining which assets are available in the release.
The limit is 60 requests per hour per IP address.
Normal GitHub Actions such as 
```yml
- uses: JamesIves/github-pages-deploy-action@v4
```

receive the `GITHUB_TOKEN` by default [via the `github.token` context](https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication).
If you don't want to use a `GITHUB_TOKEN` it is also possible to manually specify the `--url` instead of `--gh`.

## Development

Rust is used only for the test harness. Run `cargo test --locked`; tests use
local archives and a mock HTTP client, so they do not download release assets or
need a GitHub token. They cover checksum failures, archive formats, multiple
output names, platform selection, unsafe paths, and CLI validation. CI runs the
harness on x64 and ARM64 Linux, macOS, and Windows, and also checks macOS Bash
3.2. Run `shellcheck jas` and `cargo fmt --all -- --check` for linting.

On Windows, the harness locates Git Bash through the Git installation, so
`cargo test --locked` works without extra setup. Git for Windows must be installed.
`JAS_TEST_BASH` is an optional override for selecting a specific Bash executable,
for example `JAS_TEST_BASH=/bin/bash cargo test --locked` on macOS or Linux.

## Background

This tool is primarily intended to be used in CI as a workaround for GitHub Actions's poor security guarantees.
For example, recently the `tj-actions/changed-files` Action caused many repositories to leak their secrets.
As with many problems, multiple things have to go wrong for this to happen.
First, someone gained access to `changed-files` and [inserted malicious code](https://github.com/tj-actions/changed-files/issues/2464#issuecomment-2727020537).
Then, the attacker was able to not only change the latest release, but also tags [for older releases](https://github.com/tj-actions/changed-files/issues/2463).
This is a fundamental problem for GitHub Actions.
It is possible to retroactively change the tags.
So even clients that pinned to an older version of `changed-files` would start using the malicious version.
For example, `changed-files` was at v46.0.1 at the time of the attack.
This means that if you would use

```yml
- uses: tj-actions/changed-files@46
```

then this would be interpreted by GitHub as `46.0.1` and you would automatically start using the malicious version.
However, even if you pinned to an older release like `46.0.0`:

```yml
- uses: tj-actions/changed-files@46.0.0
```

you would still not be safe since the attacker has changed the tag for `46.0.0`.

The new/old way to solve this is to use explicit commit hashes.
For example, `changed-files` now advises to use this:

```yml
- uses: tj-actions/changed-files@823fcebdb31bb35fdf2229d9f769b400309430d0 # v46
```

Pinning is a lot safer, but unfortunately Git at the time of writing still uses SHA-1. Although Git runs a hardened version of SHA-1, [git-scm.com states that](https://git-scm.com/docs/hash-function-transition):

> Thus it’s considered prudent to move past any variant of SHA-1 to a new hash.
> There's no guarantee that future attacks on SHA-1 won’t be published in the future, and those attacks may not have viable mitigations.

Furthermore, I personally dislike this hash pinning approach since it doesn't specify the version. 
That's why it is very common to see the version number specified in the comment, as is done here.
The problem with this approach is that the comment can now become out of sync with the actual version.

This tool is a workaround for this problem for situations where executables (binaries or scripts) are available.
It turns the syntax into:

```yml
- run: >
    jas install
    --gh crate-ci/typos@v1.31.1
    --sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
```

Now it's clear which version is being used.
When it downloads a binary, it will verify the SHA-256 checksum.
If this checksum does not match, the tool will fail.

Unlike the GitHub Actions syntax, the version cannot become out of sync with the hash.
Also, with this method, you know exactly what you run.
With GitHub Actions, even when the commit hash is pinned, the dependencies could still change if I understand correctly.

## How does this compare to `cargo install`

Compared to GitHub Releases, `cargo install` already provides much better security guarantees.
As far as I understand, unlike with GitHub Releases it is not possible to change published versions after publication.
So if an attacker manages to publish a new malicious version on `crates.io`, then this would not affect older versions pinned to an explicit version.
Instead, the attacker would need to hack `crates.io` itself to change older versions.
Depending on the threat model it can still be useful to confirm the sha of course.

To answer the question, in most cases I would say that installations via `cargo install crate@x.y.z` are much safer than `uses: owner/repo@x.y.z`.
The only problem could be that compilation of the crate takes long.
`jas` avoids this problem by downloading the binaries from the release.
