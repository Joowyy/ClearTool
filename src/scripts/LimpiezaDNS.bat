@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN

echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs"
del "%TEMP%\elev.vbs" >nul 2>&1
exit /b

:ADMIN
title Limpieza Cache DNS
cls
echo.
echo  ================================================
echo           LIMPIEZA DE CACHÉ DNS
echo  ================================================
echo.

ipconfig /flushdns

echo.
echo  ================================================
echo.

if /i "%1"=="/nopause" goto :EOF
pause >nul
