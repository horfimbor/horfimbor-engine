default: precommit

precommit: test clippy
    cargo fmt

dc-start:
    docker compose up -d

dc-stop:
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