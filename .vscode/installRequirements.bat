echo "installing required cargo extentions cargo-watch and trunk"
rustup +nightly target add wasm32-unknown-unknown
cargo install --locked cargo-watch trunk
echo "Sucsesfully installed cargo-watch and trunk exiting in 5 sec"
timeout /t 5 /nobreak > nul