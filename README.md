# jas

Just an installer.

This tool is meant to be used in situations where you want to reliably install a binary.
By reliably, I mean that you want to specify the SHA-256 checksum so that you can be sure that you are using the correct binary.
I wrote this tool in response to yet another GitHub Action security issue.
See the [Background](#background) section for more details.

## Installation

```bash
cargo install --debug --git https://github.com/rikhuijzer/jas
```

## Usage

```bash
jas install \
  --gh crate-ci/typos@v1.31.1 \
  --sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
```

This is the SHA for MacOS aarch64.
To get the SHA for other platforms, you can use:

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

      - name: Install jas
        run: |
          cargo install --debug --git https://github.com/rikhuijzer/jas
          echo "$HOME/.jas/bin" >> $GITHUB_PATH

      - name: Install typos
        run: |
          jas install \
            --gh crate-ci/typos@v1.31.1 \
            --sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993

      - name: Run typos
        run: typos .
```

As stated above, the benefit of this is that you can be sure which version of the binary you are using.
If someone changes the binary, the SHA will change and your CI will fail.

## More usage examples

To install a binary from a GitHub release, you can use the following command:

```bash
jas install --gh casey/just@1.40.0
```

By default, this will store the binary in `~/.jas/bin`.
You can change this by using the `--dir` flag.

You can also specify the SHA of the release you want to install.

```bash
jas install \
  --gh casey/just@1.40.0 \
  --sha 0fb2401a46409bdf574f42f92df0418934166032ec2bcb0fc7919b7664fdcc01
```

To get this SHA, you can use:

```bash
jas sha \
  --url github.com/casey/just/releases/download/1.40.0/just-1.40.0-aarch64-apple-darwin.tar.gz
```

Or if you already have the file locally:

```bash
jas sha --path just-1.40.0-aarch64-apple-darwin.tar.gz
```

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
- name: Get changed files
  uses: tj-actions/changed-files@46.0.0
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
- run: |
    jas install \
      --gh crate-ci/typos@v1.31.1 \
      --sha f683c2abeaff70379df7176110100e18150ecd17a4b9785c32908aca11929993
```
Now it's clear which version is being used.
When it downloads a binary, it will verify the SHA-256 checksum.
If this checksum does not match, the tool will fail.

Unlike the GitHub Actions syntax, the version cannot become out of sync with the hash.
Also, with this method, you know exactly what you run.
With GitHub Actions, even when the commit hash is pinned, the dependencies could still change if I understand correctly.
