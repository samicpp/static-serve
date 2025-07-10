:: x86_64-unknown-linux-gnu  aarch64-unknown-linux-gnu  x86_64-pc-windows-gnu  x86_64-pc-windows-msvc

@REM @cargo build --target x86_64-unknown-linux-gnu --release
@REM @copy .\target\x86_64-unknown-linux-gnu\release\static_serve .\bin\x86_64-unknown-linux-gnu

@REM @cargo build --target aarch64-unknown-linux-gnu --release
@REM @copy .\target\aarch64-unknown-linux-gnu\release\static_serve .\bin\aarch64-unknown-linux-gnu

@cargo build --target x86_64-pc-windows-gnu --release
@copy .\target\x86_64-pc-windows-gnu\release\static_serve.exe .\bin\x86_64-pc-windows-gnu.exe

@cargo build --target x86_64-pc-windows-msvc --release
@copy .\target\x86_64-pc-windows-msvc\release\static_serve.exe .\bin\x86_64-pc-windows-msvc.exe
