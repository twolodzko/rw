test: basic-test integration-test

basic-test:
    cargo fmt
    cargo clippy
    cargo test

integration-test: build
    bats --pretty tests.bats

build:
    cargo build --release
    cp ./target/release/rw ./

install: test
    cargo install --path .

clean:
    rm -rf ./target
    rm -rf ./rw

lines:
    @ find . -name '*.rs' -exec wc -l {} \;
    @ find . -name '*.rs' -exec cat {} \; | wc -l
