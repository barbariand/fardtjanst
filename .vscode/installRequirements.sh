#!bin/bash
echo "installing required cargo extentions cargo-watch and trunk"
rustup +nightly target add wasm32-unknown-unknown
cargo install --locked cargo-watch trunk &

wait
echo "Sucsesfully installed cargo-watch and trunk exiting in 5 sec"
sleep 5 &
wait