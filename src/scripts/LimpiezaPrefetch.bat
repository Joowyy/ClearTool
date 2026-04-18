@echo off
setlocal enabledelayedexpansion

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Prefetch
cls
echo.
echo  [Prefetch]  C:\Windows\Prefetch
echo.
takeown /f "C:\Windows\Prefetch" /r /d y >nul 2>&1
icacls "C:\Windows\Prefetch" /grant Administrators:F /t /q >nul 2>&1
for /d %%D in ("C:\Windows\Prefetch\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "C:\Windows\Prefetch\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "C:\Windows\Prefetch" 2^>nul ^| find /c /v ""') do set count=%%A
if !count! == 0 (echo  [OK] Todo limpio.) else (echo  [!] !count! elemento(s^) en uso.)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
