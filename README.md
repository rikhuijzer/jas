# jas

Just an installer.

This tool is meant to be used in situations where you want to install a binary in a reliable way, that is, by specifying the SHA-256 checksum.
The checksum ensures that the binary is exactly the one you expect it to be.
See the [Background](#background) section for more details.

## Installation

```bash
cargo install --debug jas
```

and add `~/.jas/bin` to your PATH.

## Usage

To install Typos from GitHub into `~/.jas/bin`, you can use:

```bash
jas install \
--gh crate-ci/typos@v1.31.1 \
--sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
```

This command uses the SHA for the MacOS aarch64 release.
To get the SHA for other platforms, you can use `sha --url`.
For example,

```bash
jas sha \
--url github.com/crate-ci/typos/releases/download/v1.31.1/typos-v1.31.1-x86_64-unknown-linux-musl.tar.gz
```

## Usage in GitHub Actions

For example, to install and run [`typos`](https://github.com/crate-ci/typos) v1.31.1, you can use the following job in your GitHub Actions workflow:

```yaml
jobs:
  typos:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    timeout-minutes: 10

    steps:
      - uses: actions/checkout@v4
      - run: cargo install --debug jas
      - run: >
          jas install
          --gh crate-ci/typos@v1.31.1
          --sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
          --gh-token ${{ secrets.GITHUB_TOKEN }}
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

## Background

This tool is primarily intended to be used in CI as a workaround for GitHub Actions's poor security guarantees.
For example, recently the `tj-actions/changed-files` Action caused many repositories to leak their secrets.
As with many problems, multiple things have to go wrong for this to happen.
First, someone gained access to `changed-files` and [inserted malicious code into it](https://github.com/tj-actions/changed-files/issues/2464#issuecomment-2727020537).
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

This of course is better, but I personally dislike using commit hashes.
The main problem is that it's hard to tell which version is being used, which is why it is typical to write a comment with the version number.

This tool is a workaround for this problem for situations where binaries are available.
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
