@echo off
setlocal enabledelayedexpansion

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN

echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs"
del "%TEMP%\elev.vbs" >nul 2>&1
exit /b

:ADMIN
title Limpieza Cache Microsoft Store
cls
set pkg=%LOCALAPPDATA%\Packages\Microsoft.WindowsStore_8wekyb3d8bbwe
set totalRestantes=0

echo.
echo  ================================================
echo      LIMPIEZA DE CACHÉ — MICROSOFT STORE
echo  ================================================
echo.

:: Verificar que el paquete existe
if not exist "%pkg%" (
    echo  [!] Microsoft Store no encontrada en este equipo.
    echo.
    if /i "%1"=="/nopause" goto :EOF
    pause >nul
    exit /b
)

echo  [1/3] LocalCache...
for /d %%D in ("%pkg%\LocalCache\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "%pkg%\LocalCache\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "%pkg%\LocalCache" 2^>nul ^| find /c /v ""') do set count=%%A
set /a totalRestantes+=count
if !count! == 0 (echo         [OK] Todo limpio.) else (echo         [!] !count! elemento(s^) en uso.)
echo.

echo  [2/3] INetCache...
for /d %%D in ("%pkg%\AC\INetCache\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "%pkg%\AC\INetCache\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "%pkg%\AC\INetCache" 2^>nul ^| find /c /v ""') do set count=%%A
set /a totalRestantes+=count
if !count! == 0 (echo         [OK] Todo limpio.) else (echo         [!] !count! elemento(s^) en uso.)
echo.

echo  [3/3] Temp...
for /d %%D in ("%pkg%\AC\Temp\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "%pkg%\AC\Temp\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "%pkg%\AC\Temp" 2^>nul ^| find /c /v ""') do set count=%%A
set /a totalRestantes+=count
if !count! == 0 (echo         [OK] Todo limpio.) else (echo         [!] !count! elemento(s^) en uso.)
echo.

echo  ================================================
echo   Nota: La Microsoft Store NO se abrira.
echo   (metodo directo sin WSReset.exe)
if !totalRestantes! == 0 (
    echo   RESULTADO: Cache limpiada correctamente.
) else (
    echo   RESULTADO: !totalRestantes! elemento(s^) bloqueados por procesos activos.
)
echo  ================================================
echo.

if /i "%1"=="/nopause" goto :EOF
pause >nul
