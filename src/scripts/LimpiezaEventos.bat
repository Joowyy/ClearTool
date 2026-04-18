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
set borrados=0
set errores=0

echo.
echo  ================================================
echo        LIMPIEZA DE REGISTROS DE EVENTOS
echo  ================================================
echo.
echo  Obteniendo lista de registros...
echo.

for /f "tokens=*" %%G in ('wevtutil el 2^>nul') do (
    wevtutil cl "%%G" >nul 2>&1
    if !errorlevel! == 0 (
        echo  [OK] %%G
        set /a borrados+=1
    ) else (
        echo  [--] %%G  ^(protegido o en uso^)
        set /a errores+=1
    )
)

echo.
echo  ================================================
echo   Limpiados : !borrados! registros
echo   Omitidos  : !errores! registros ^(protegidos por el sistema^)
echo  ================================================
echo.

if /i "%1"=="/nopause" goto :EOF
pause >nul
