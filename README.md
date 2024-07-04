# cookiecluster

Clusters today are boring. Boring is good.

## Installation

[Archives of pre-compiled binaries for `cookiecluster` are available for Windows, macOS and Linux.](https://github.com/clowdhaus/cookiecluster/releases)

### Homebrew (macOS and Linux)

```sh
brew install clowdhaus/taps/cookiecluster
```

### Cargo (rust)

```sh
cargo install cookiecluster
```

### Source

`cookiecluster` is written in Rust, so you'll need to grab a [Rust installation](https://www.rust-lang.org/) in order to compile it.
`cookiecluster` compiles with Rust 1.79.0 (stable) or newer. In general, `cookiecluster` tracks the latest stable release of the Rust compiler.

To build `cookiecluster`:

```sh
git clone https://github.com/clowdhaus/cookiecluster
cd cookiecluster
cargo build --release
./target/release/cookiecluster --version
0.1.0
```

## Local Development

`cookiecluster` uses Rust stable for production builds, but nightly for local development for formatting and linting. It is not a requirement to use nightly, but if running `fmt` you may see a few warnings on certain features only being available on nightly.

Build the project to pull down dependencies and ensure everything is setup properly:

```sh
cargo build
```

To format the codebase:

If using nightly to use features defined in [rustfmt.toml](rustfmt.toml), run the following:

```sh
cargo +nightly fmt --all
```

If using stable, run the following:

```sh
cargo fmt --all
```

To execute lint checks:

```sh
cargo clippy --all-targets --all-features
```

To run `cookiecluster` locally for development:

```sh
cargo run
```

### Running Tests

To execute the tests provided, run the following from the project root directory:

```sh
cargo test --all
```
