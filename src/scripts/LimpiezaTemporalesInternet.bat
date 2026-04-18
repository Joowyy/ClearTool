@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Temporales de Internet
cls
set inetPath=%LOCALAPPDATA%\Microsoft\Windows\INetCache
echo.
echo  [Temporales Internet]  %inetPath%
echo.
if not exist "%inetPath%" (
    echo  [--] Ruta no encontrada, omitida.
) else (
    for /d %%D in ("%inetPath%\*") do rd /s /q "%%D" >nul 2>&1
    del /f /q "%inetPath%\*" >nul 2>&1
    echo  [OK] Cache de Edge / IE limpiada.
)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
