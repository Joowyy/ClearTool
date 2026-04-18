@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Optimizacion de Distribucion
cls
echo.
echo  [Optimizacion de Distribucion]  Cache P2P de Windows Update
echo.
powershell -Command "Delete-DeliveryOptimizationCache -Force -ErrorAction SilentlyContinue"
if %errorLevel% == 0 (echo  [OK] Cache eliminada.) else (echo  [!] Comando no disponible en este sistema.)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
