@echo off

net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN
echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs" & del "%TEMP%\elev.vbs" >nul 2>&1 & exit /b

:ADMIN
title Limpieza — Microsoft Defender
cls
set defPath=C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory
echo.
echo  [Defender]  Historial de deteccion no critico
echo.
if not exist "%defPath%" (
    echo  [--] Ruta no encontrada, omitida.
) else (
    takeown /f "%defPath%" /r /d y >nul 2>&1
    icacls "%defPath%" /grant Administrators:F /t /q >nul 2>&1
    for /d %%D in ("%defPath%\*") do rd /s /q "%%D" >nul 2>&1
    del /f /q "%defPath%\*" >nul 2>&1
    echo  [OK] Historial limpiado.
)
echo.
if /i "%1"=="/nopause" goto :EOF
pause >nul
