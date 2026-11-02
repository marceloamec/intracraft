@echo off
rem Opens the reference layout in the Discrete simulation module, running, on the floor plan.
rem The first time it compiles the optimised program (several minutes); afterwards it opens at once.
cd /d "%~dp0..\platform"
cargo run --release --offline -p intracraft -- "%~dp0..\packages\scenarios\scenarios\layouts\reference-all-five.intracraft.json" --module discrete --play
if errorlevel 1 pause
