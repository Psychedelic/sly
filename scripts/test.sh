#!/bin/sh
cd ..
# cargo run -- --name da --template fungible_token
cargo build
cd target/debug/ || exit
# ./sly new --name sly_ft --template fungible_token
# ./sly new --name sly_nft --template non_fungible_token
./sly new --template rust_backend ../../../sly_rust
# ./sly new --name sly_default
cd sly_rust || exit
../sly build --with-mode release --all