
build:
    cargo build --release

run: build
    ./maelstrom/maelstrom test -w echo --bin target/release/distributed-systems --node-count 1 --time-limit 10
