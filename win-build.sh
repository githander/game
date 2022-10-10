mkdir win-out

cargo build --target i686-pc-windows-gnu --release
cp target/i686-pc-windows-gnu/release/game.exe win-out/
cp assets win-out/assets -r -T
