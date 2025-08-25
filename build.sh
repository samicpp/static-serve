# x86_64-unknown-linux-gnu  aarch64-unknown-linux-gnu  x86_64-pc-windows-gnu  x86_64-pc-windows-msvc

cargo build --target x86_64-unknown-linux-gnu --release
cp ./target/x86_64-unknown-linux-gnu/release/static_serve ./bin/x86_64-unknown-linux-gnu

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
cargo build --target aarch64-unknown-linux-gnu --release
cp ./target/aarch64-unknown-linux-gnu/release/static_serve ./bin/aarch64-unknown-linux-gnu

cargo build --target x86_64-pc-windows-gnu --release
cp ./target/x86_64-pc-windows-gnu/release/static_serve.exe ./bin/x86_64-pc-windows-gnu.exe

# cargo build --target x86_64-pc-windows-msvc --release
# cp ./target/x86_64-pc-windows-msvc/release/static_serve.exe ./bin/x86_64-pc-windows-msvc.exe
