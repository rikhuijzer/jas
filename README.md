# jas

Just an installer.

This tool is meant to be used in situations where you want to reliably install a binary.
By reliably, I mean that you want to specify the SHA-256 checksum so that the binary can not be changed without you noticing.

This tool is primarly intended to be used in CI as a workaround for GitHub Actions's poor security guarantees.
For example, recently the `tj-actions/changed-files` Action caused many repositories to leak their secrets.
As with many problems, multiple things have to go wrong for this to happen.
First, someone gained access to `changed-files` and [inserted malicious code into it](https://github.com/tj-actions/changed-files/issues/2464#issuecomment-2727020537).
Then, the attacker was able to not only change the latest release, but also tags [for older releases](https://github.com/tj-actions/changed-files/issues/2463).
This is a fundamental problem for GitHub Actions.
It is possible to retroactively change the tags.
So even clients that pinned to a specific version of `changed-files` would start using the malicious version.

This tool is a workaround for this problem.
When it downloads a binary, it will verify the SHA-256 checksum.
If this checksum does not match, the tool will fail and the CI will fail.
Apart from security benefits, this also ensures that the version that you are using is not quietly updated when you least expect it.

## Usage in GitHub Actions

For example, to install Typos v1.31.1, you can use the following job in your GitHub Actions workflow:

```yaml
jobs:
  typos:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    timeout-minutes: 10

    steps:
      - uses: actions/checkout@v4
      - run: cargo install --debug --git https://github.com/rikhuijzer/jas
      - run: echo "$HOME/.jas/bin" >> $GITHUB_PATH
      - run: jas install \
            --gh crate-ci/typos@v1.31.1 \
            --sha 399e6f883b8d97f822e8b9662d5377820d46f60dd33e95881e3173cebea6d70c
      - run: typos .
```

As stated above, the benefit of this is that you can be sure which version of the binary you are using.
If someone changes the binary, the SHA will change and your CI will fail.

## Usage

To install a binary from a GitHub release, you can use the following command:

```bash
jas install --gh casey/just@1.40.0
```

By default, this will store the binary in `~/.jas/bin`.
You can change this by using the `--output` flag.

You can also specify the SHA of the release you want to install.

```bash
jas install --gh casey/just@1.40.0 --sha256 1234567890
```

To get this SHA, you can use:

```bash
jas sha --gh casey/just@1.40.0
```

Or if you already have the file locally:

```bash
jas sha --path just-1.40.0.tar.gz
```

## Installation

This tool is mostly meant to be used in CI.

```bash
cargo install jas
```
