@echo off
rem Opens the mixed example plant in the Discrete simulation module, running: the floor plan and the 3D view side by side.
rem The first time it compiles the optimised programs (several minutes); afterwards it opens at once.
cd /d "%~dp0..\platform"
cargo build --release --offline -p intracraft -p intracraft-viewer || goto failed
start "" "target\release\intracraft.exe" "%~dp0..\examples\layouts\mixed-plant.intracraft.json" --module discrete --play --split
exit /b 0
:failed
pause
