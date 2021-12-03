#!/bin/sh
cd ..
cargo build
cd target/debug/ || exit
./sly new --name da --template fungible_token