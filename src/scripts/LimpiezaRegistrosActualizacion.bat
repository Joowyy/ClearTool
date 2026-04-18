@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Registros de Actualizacion de Windows
cls
echo.
echo  [Registros Actualizacion]  C:\Windows\Logs\WindowsUpdate
echo.
if not exist "C:\Windows\Logs\WindowsUpdate" (
    echo  [--] Ruta no encontrada, omitida.
) else (
    del /f /q "C:\Windows\Logs\WindowsUpdate\*.etl" >nul 2>&1
    del /f /q "C:\Windows\Logs\WindowsUpdate\*.log" >nul 2>&1
    echo  [OK] Logs de Windows Update eliminados.
)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
