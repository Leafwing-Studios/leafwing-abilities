# About

`leafwing-abilities` is an opinionated, ready-to-use Bevy library and plugin for handling abilities.
It features:

- cooldown tracking
- resource management
- a battle-tested, coherent structure for writing your own ability logic

Roleplaying games, and particularly Diablo-like action RPGs are the primary target, but you may find it useful for platformers, turn-based RPGs, real-time strategy games and games looking for MOBA-style gameplay.

## Instructions

Finally:

1. Rename the lib and bin in `Cargo.toml` (and all imports to ensure your code compiles).
2. Double check that the LICENSE matches your intent.
3. Update this README to match your project, modifying `About`, `Getting Started` and other sections as needed.

### Running examples

To run an example, use `cargo run --example_name`, where `example_name` is the file name of the example without the `.rs` extension.

This repo is set up to always build with full optimizations, so there's no need for a `--release` flag in most cases.
Dynamic linking is enabled to ensure build times stay snappy.

## Contributing

This repository is open to community contributions!
There are a few options if you'd like to help:

1. File issues for bugs you find or new features you'd like.
2. Read over and discuss issues, then make a PR that fixes them. Use "Fixes #X" in your PR description to automatically close the issue when the PR is merged.
3. Review existing PRs, and leave thoughtful feedback. If you think a PR is ready to merge, hit "Approve" in your review!

Any contributions made are provided under the licenses (standard MIT + Apache) listed in this repo at the time of their contribution, and do not require separate attribution.

### Testing

1. Use doc tests aggressively to show how APIs should be used.
You can use `#` to hide a setup line from the doc tests.
2. Unit test belong near the code they are testing. Use `#[cfg(test)]` on the test module to ignore it during builds, and `#[test]` on the test functions to ensure they are run.
3. Integration tests should be stored in the top level `tests` folder, importing functions from `lib.rs`.

Use `cargo test` to run all tests.

### CI

The CI will:

1. Ensure the code is formatted with `cargo fmt`.
2. Ensure that the code compiles.
3. Ensure that (almost) all `clippy` lints pass.
4. Ensure all tests pass on Windows, MacOS and Ubuntu.

Check this locally with:

1. `cargo run -p ci`
2. `cargo test --workspace`

To manually rerun CI:

1. Navigate to the `Actions` tab.
2. Use the dropdown menu in the CI run of interest and select "View workflow file".
3. In the top-right corner, select "Rerun workflow".

### Documentation

Reference documentation is handled with standard Rust doc strings.
Use `cargo doc --open` to build and then open the docs.

Design docs (or other book-format documentation) is handled with [mdBook](https://rust-lang.github.io/mdBook/index.html).
Install it with `cargo install mdbook`, then use `mdbook serve --open` to launch the docs.

### Benchmarking

To run the benchmarks, use `cargo bench`.

For more documentation on making your own benchmarks, check out [criterion's docs](https://bheisler.github.io/criterion.rs/book/index.html).
