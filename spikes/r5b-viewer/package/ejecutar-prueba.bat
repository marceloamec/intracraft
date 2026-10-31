@echo off
rem Prueba R5b: mide los dos visores 3D, uno tras otro, y escribe resultados.txt.
rem Cierre los demas programas y enchufe el portatil a la corriente antes de empezar.
chcp 65001 >nul
cd /d "%~dp0"
echo.
echo Prueba R5b - visor 3D de la simulacion
echo ======================================
echo Se abriran dos ventanas, una tras otra, durante 30 segundos cada una.
echo No toque el raton ni el teclado mientras se mide.
echo.
pause
del /q result-egui.json result-bevy.json resultados.txt 2>nul
echo Midiendo la vista propia tipo BIMCraft...
r5b-viewer-egui.exe --bench 30
echo Midiendo el visor Bevy...
r5b-viewer-bevy.exe --bench 30
echo.
r5b-report.exe
echo.
echo Abra resultados.txt y copie su contenido en el chat.
pause
