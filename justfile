default: precommit

precommit: test clippy
    cargo fmt

start:
    docker compose up -d

stop:
    docker compose down

test:
    cargo test

doc projet:
    cargo doc -p horfimbor-{{projet}} --open

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