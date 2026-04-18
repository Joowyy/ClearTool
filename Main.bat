@echo off
setlocal enabledelayedexpansion

:: ── Elevacion a Administrador ────────────────────────────────────────
net session >nul 2>&1
if %errorLevel% == 0 goto :ADMIN

echo Set UAC = CreateObject("Shell.Application") > "%TEMP%\elev.vbs"
echo UAC.ShellExecute "%~s0", "", "", "runas", 1 >> "%TEMP%\elev.vbs"
cscript //nologo "%TEMP%\elev.vbs"
del "%TEMP%\elev.vbs" >nul 2>&1
exit /b

:ADMIN
title Limpieza General del PC
cls
echo.
echo  ================================================
echo            LIMPIEZA GENERAL DEL PC
echo  ================================================
echo.
echo   Este proceso ejecutara en orden:
echo.
echo     [BLOQUE 1]  Archivos temporales y cache
echo                 ^(%%TEMP%%, Windows\Temp, Prefetch^)
echo.
echo     [BLOQUE 2]  Registros del Visor de Eventos
echo.
echo  ================================================
echo.
echo  Pulsa una tecla para iniciar...
pause >nul

:: ── BLOQUE 1 ─────────────────────────────────────────────────────────
cls
echo.
echo  ════════════════════════════════════════════════
echo   BLOQUE 1 / 2  ^|  Archivos Temporales
echo  ════════════════════════════════════════════════
echo.
call "%~dp0scripts\LimpiezaPC.bat" /nopause

:: ── Transicion ───────────────────────────────────────────────────────
echo.
echo  ------------------------------------------------
echo   Bloque 1 completado.
echo   Pulsa una tecla para continuar con el Bloque 2...
echo  ------------------------------------------------
pause >nul

:: ── BLOQUE 2 ─────────────────────────────────────────────────────────
cls
echo.
echo  ════════════════════════════════════════════════
echo   BLOQUE 2 / 2  ^|  Registros de Eventos
echo  ════════════════════════════════════════════════
echo.
call "%~dp0scripts\LimpiezaEventos.bat" /nopause

:: ── Fin ──────────────────────────────────────────────────────────────
echo.
echo  ════════════════════════════════════════════════
echo         LIMPIEZA GENERAL COMPLETADA
echo  ════════════════════════════════════════════════
echo.
echo  Pulsa una tecla para cerrar.
pause >nul
