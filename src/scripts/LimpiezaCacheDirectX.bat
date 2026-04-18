@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Cache DirectX
cls
set dxPath=%LOCALAPPDATA%\D3DSCache
echo.
echo  [Cache DirectX]  %dxPath%
echo.
if not exist "%dxPath%" (
    echo  [--] Ruta no encontrada, omitida.
) else (
    for /d %%D in ("%dxPath%\*") do rd /s /q "%%D" >nul 2>&1
    del /f /q "%dxPath%\*" >nul 2>&1
    echo  [OK] Cache de shaders DirectX limpiada.
)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
