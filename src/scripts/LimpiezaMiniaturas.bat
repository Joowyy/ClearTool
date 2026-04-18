@echo off
setlocal enabledelayedexpansion

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Miniaturas
cls
set explorer_dir=%LOCALAPPDATA%\Microsoft\Windows\Explorer
set count=0
echo.
echo  [Miniaturas]  %explorer_dir%
echo.
for %%F in ("%explorer_dir%\thumbcache_*.db") do (
    del /f /q "%%F" >nul 2>&1 && set /a count+=1
)
for %%F in ("%explorer_dir%\iconcache_*.db") do (
    del /f /q "%%F" >nul 2>&1 && set /a count+=1
)
echo  [OK] !count! archivo(s^) de miniatura eliminados.
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
