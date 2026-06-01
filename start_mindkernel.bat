@echo off
set RUST_LOG=info
start /B "" "D:\mindkernel\target\release\mindkernel.exe" > "D:\mindkernel\mindkernel_prod.log" 2>&1
echo mindkernel started with PID: %ERRORLEVEL%
