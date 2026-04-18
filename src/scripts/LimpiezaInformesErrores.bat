@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Informes de Errores de Windows
cls
echo.
echo  [Informes de Errores]  WER — Windows Error Reporting
echo.
for /d %%D in ("%LOCALAPPDATA%\Microsoft\Windows\WER\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "%LOCALAPPDATA%\Microsoft\Windows\WER\*" >nul 2>&1
echo  [OK] WER usuario limpiado.

for /d %%D in ("C:\ProgramData\Microsoft\Windows\WER\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "C:\ProgramData\Microsoft\Windows\WER\*" >nul 2>&1
echo  [OK] WER sistema limpiado.
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
