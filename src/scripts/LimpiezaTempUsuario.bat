@echo off
setlocal enabledelayedexpansion

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Temp Usuario
cls
echo.
echo  [Temp Usuario]  %TEMP%
echo.
takeown /f "%TEMP%" /r /d y >nul 2>&1
icacls "%TEMP%" /grant Administrators:F /t /q >nul 2>&1
for /d %%D in ("%TEMP%\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "%TEMP%\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "%TEMP%" 2^>nul ^| find /c /v ""') do set count=%%A
if !count! == 0 (echo  [OK] Todo limpio.) else (echo  [!] !count! elemento(s^) en uso.)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
