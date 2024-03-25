precommit: clippy
    cargo fmt

clippy:
    cargo clippy --all-features -- \
    -D clippy::correctness \
    -D clippy::complexity \
    -D clippy::pedantic \
    -D clippy::nursery \
    -D clippy::perf \
    -D clippy::all \
    -D clippy::expect_used \
    -W clippy::cargo