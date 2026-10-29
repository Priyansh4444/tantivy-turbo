# Contributing to tantivy-turbo

Issues, questions, documentation improvements and pull requests are welcome.
You do not need to ask permission for a small fix. For a larger feature or a
change to scoring, index formats or search behavior, opening an issue first
helps agree on the intended behavior and avoid duplicated work.

## Get started

1. Fork the repository and create a branch from `main`.
2. Make a focused change and add or update tests when behavior changes.
3. Run the relevant checks below, then open a pull request using the template.
   Draft pull requests are welcome if you would like feedback early.

Rust 1.87 or newer is the declared minimum; CI also uses current stable and
nightly Rust. From the repository root, useful checks include:

```sh
cargo +nightly fmt --all -- --check
cargo +stable test --workspace
cargo +stable test --release --test pruning_exactness
```

The workspace supports multiple feature combinations; the pull request CI
runs its feature matrix, formatting, compilation, linting and the release
search-correctness test. Run the most relevant checks locally and describe
anything you could not run in the pull request. The demo is a separate Cargo
project; its build and render instructions are in [demo/README.md](demo/README.md).

## Correctness and performance

Search optimizations must preserve the intended matches, ranking, scores and
counts. For pruning changes, compare against exhaustive scoring, including
edge cases such as deletions, repeated terms and non-default scoring parameters.
For a performance claim, describe the workload, hardware, build, baseline,
measurement method and correctness checks. Report regressions and relevant
limitations as well as wins. [BENCHMARKS.md](BENCHMARKS.md) explains the
existing measurements and links to their evidence.

Please report security vulnerabilities privately as described in
[SECURITY.md](SECURITY.md), rather than opening a public issue.
