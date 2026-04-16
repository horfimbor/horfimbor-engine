default: precommit

precommit: test clippy
    cargo fmt

alias dc-db := dc-up
dc-up:
    docker compose up -d

dc-down:
    docker compose down

test:
    cargo test

doc projet:
    cargo doc -p horfimbor-{{projet}} --open

clippy:
    cargo clippy --all-features -- \
    -W clippy::correctness \
    -W clippy::complexity \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::perf \
    -W clippy::all \
    -W clippy::expect_used \
    -W clippy::cargo \
    -A clippy::multiple_crate_versions \
    -D warnings