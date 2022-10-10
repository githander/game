mkdir linux-out

cargo build --release
cp target/release/game linux-out/
cp assets linux-out/assets -r -T
