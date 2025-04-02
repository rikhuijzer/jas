# jas

Just an installer.

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
