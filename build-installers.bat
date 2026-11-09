@echo off
rem Regenera el ejecutable y los instaladores (NSIS y MSI). Doble clic o ejecutar desde cualquier carpeta.
setlocal
cd /d "%~dp0"

echo === Instalando dependencias de npm si faltan ===
call npm install --no-audit --no-fund
if errorlevel 1 goto :fail

echo.
echo === Compilando (release + instaladores). Tarda unos minutos ===
call npm run tauri -- build
if errorlevel 1 goto :fail

echo.
echo === Listo ===
echo Ejecutable: %~dp0src-tauri\target\release\miagendaapp.exe
echo NSIS:       %~dp0src-tauri\target\release\bundle\nsis\
echo MSI:        %~dp0src-tauri\target\release\bundle\msi\
explorer "%~dp0src-tauri\target\release\bundle"
pause
exit /b 0

:fail
echo.
echo *** La compilacion ha fallado. Revisa los mensajes de arriba. ***
pause
exit /b 1
