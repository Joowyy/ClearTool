@echo off
setlocal enabledelayedexpansion

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Temporales Instalacion Windows
cls
echo.
echo  [Instalacion Windows]  $WINDOWS.~BT  /  $WINDOWS.~WS
echo.

if exist "C:\$WINDOWS.~BT" (
    takeown /f "C:\$WINDOWS.~BT" /r /d y >nul 2>&1
    icacls "C:\$WINDOWS.~BT" /grant Administrators:F /t /q >nul 2>&1
    rd /s /q "C:\$WINDOWS.~BT" >nul 2>&1
    echo  [OK] $WINDOWS.~BT eliminado.
) else (
    echo  [--] $WINDOWS.~BT no existe.
)

if exist "C:\$WINDOWS.~WS" (
    takeown /f "C:\$WINDOWS.~WS" /r /d y >nul 2>&1
    icacls "C:\$WINDOWS.~WS" /grant Administrators:F /t /q >nul 2>&1
    rd /s /q "C:\$WINDOWS.~WS" >nul 2>&1
    echo  [OK] $WINDOWS.~WS eliminado.
) else (
    echo  [--] $WINDOWS.~WS no existe.
)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
