@echo off
rem Opens IntraCraft's 3D window (Bevy, with shadows) on the mixed example plant, its physics run (Avian) playing.
rem Drop another layout (.intracraft.json) on this file to open that one instead.
cd /d "%~dp0..\platform"
cargo build --release --offline -p intracraft-viewer || goto failed
set "layout=%~1"
if "%layout%"=="" set "layout=%~dp0..\examples\layouts\mixed-plant.intracraft.json"
start "" "target\release\intracraft-viewer.exe" "%layout%" --module physics --play
exit /b 0
:failed
pause
