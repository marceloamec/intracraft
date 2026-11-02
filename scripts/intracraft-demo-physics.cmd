@echo off
rem Opens the reference layout in the Physics simulation module, running, in the 3D view.
rem The first time it compiles the optimised program (several minutes); afterwards it opens at once.
cd /d "%~dp0..\platform"
cargo run --release --offline -p intracraft -- "%~dp0..\packages\scenarios\scenarios\layouts\reference-all-five.intracraft.json" --module physics --view 3d --play
if errorlevel 1 pause
