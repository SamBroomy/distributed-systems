build TARGET:
    cargo build --release --bin {{ TARGET }}

run-maelstrom TARGET *FLAGS: (build TARGET)
    ./maelstrom/maelstrom test -w {{ TARGET }} --bin target/release/{{ TARGET }} {{ FLAGS }}

run-echo:
    @just run-maelstrom echo --node-count 1 --time-limit 10

run-unique-ids:
    @just run-maelstrom unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

run-all: run-echo run-unique-ids