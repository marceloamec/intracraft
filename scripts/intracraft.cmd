@echo off
rem Opens IntraCraft. Drop a layout (.intracraft.json) or a project (.bimcraft) on this file to open it.
rem The first time it compiles the optimised program (several minutes); afterwards it opens at once.
cd /d "%~dp0..\platform"
cargo run --release --offline -p intracraft -- %*
if errorlevel 1 pause
