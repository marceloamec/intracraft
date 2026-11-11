@echo off
rem Pallets running off open ends and falling (no sinks): IntraCraft's 3D window, physics run (Avian) at x2.
cd /d "%~dp0..\platform"
cargo build --release --offline -p intracraft-viewer || goto failed
start "" "target\release\intracraft-viewer.exe" "%~dp0..\examples\layouts\pallet-drop.intracraft.json" --module physics --play --speed 2
exit /b 0
:failed
pause
