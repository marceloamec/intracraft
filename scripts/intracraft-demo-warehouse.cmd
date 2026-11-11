@echo off
rem Opens the example high-bay warehouse in the Discrete simulation module, running: the floor plan and the 3D view side
rem by side, the stacker cranes storing and retrieving pallets.
rem The first time it compiles the optimised programs (several minutes); afterwards it opens at once.
cd /d "%~dp0..\platform"
cargo build --release --offline -p intracraft -p intracraft-viewer || goto failed
start "" "target\release\intracraft.exe" "%~dp0..\examples\layouts\hbw-2-aisles-direct.intracraft.json" --module discrete --play --split
exit /b 0
:failed
pause
