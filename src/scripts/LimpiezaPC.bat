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
set totalRestantes=0

echo.
echo  ================================================
echo          LIMPIEZA DE ARCHIVOS TEMPORALES
echo  ================================================
echo.

echo  [1/3] %TEMP%
echo         Forzando permisos...
takeown /f "%TEMP%" /r /d y >nul 2>&1
icacls "%TEMP%" /grant Administrators:F /t /q >nul 2>&1
echo         Borrando...
for /d %%D in ("%TEMP%\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "%TEMP%\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "%TEMP%" 2^>nul ^| find /c /v ""') do set count=%%A
set /a totalRestantes+=count
if !count! == 0 (echo         [OK] Todo limpio.) else (echo         [!] !count! elemento(s^) en uso.)
echo.

echo  [2/3] C:\Windows\Temp
echo         Forzando permisos...
takeown /f "C:\Windows\Temp" /r /d y >nul 2>&1
icacls "C:\Windows\Temp" /grant Administrators:F /t /q >nul 2>&1
echo         Borrando...
for /d %%D in ("C:\Windows\Temp\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "C:\Windows\Temp\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "C:\Windows\Temp" 2^>nul ^| find /c /v ""') do set count=%%A
set /a totalRestantes+=count
if !count! == 0 (echo         [OK] Todo limpio.) else (echo         [!] !count! elemento(s^) en uso.)
echo.

echo  [3/3] C:\Windows\Prefetch
echo         Forzando permisos...
takeown /f "C:\Windows\Prefetch" /r /d y >nul 2>&1
icacls "C:\Windows\Prefetch" /grant Administrators:F /t /q >nul 2>&1
echo         Borrando...
for /d %%D in ("C:\Windows\Prefetch\*") do rd /s /q "%%D" >nul 2>&1
del /f /q "C:\Windows\Prefetch\*" >nul 2>&1
set count=0
for /f %%A in ('dir /b /s "C:\Windows\Prefetch" 2^>nul ^| find /c /v ""') do set count=%%A
set /a totalRestantes+=count
if !count! == 0 (echo         [OK] Todo limpio.) else (echo         [!] !count! elemento(s^) en uso.)
echo.

echo  ================================================
if !totalRestantes! == 0 (
    echo   RESULTADO: Limpieza total completada.
) else (
    echo   RESULTADO: !totalRestantes! elemento(s^) bloqueados por procesos activos.
)
echo  ================================================
echo.

if /i "%1"=="/nopause" goto :EOF
pause >nul
