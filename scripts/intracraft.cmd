@echo off
rem Opens IntraCraft. Drop a layout (.intracraft.json) or a project (.bimcraft) on this file to open it.
rem The first time it compiles the optimised programs, IntraCraft and its 3D window (several minutes); afterwards it opens at once.
cd /d "%~dp0..\platform"
cargo build --release --offline -p intracraft -p intracraft-viewer || goto failed
start "" "target\release\intracraft.exe" %*
exit /b 0
:failed
pause
