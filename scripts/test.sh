#!/bin/sh
cd ..
# cargo run -- --name da --template fungible_token
cargo build
cd target/debug/ || exit
./sly new --name da --template fungible_token