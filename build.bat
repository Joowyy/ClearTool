@echo off
title ClearTool PC — Build
cls
echo.
echo  ================================================
echo    ClearTool PC  —  Compilar a .exe
echo  ================================================
echo.

python --version >nul 2>&1
if %errorLevel% neq 0 (
    echo  [ERROR] Python no esta instalado o no esta en el PATH.
    echo  Descargalo desde https://python.org
    pause
    exit /b
)

echo  Instalando / verificando PyInstaller...
pip install pyinstaller --quiet

echo.
echo  Compilando...
pyinstaller --onefile --noconsole ^
    --name "ClearToolPC" ^
    --distpath "%~dp0dist" ^
    --workpath "%~dp0build_tmp" ^
    --specpath "%~dp0build_tmp" ^
    "%~dp0main.py"

echo.
if exist "%~dp0dist\ClearToolPC.exe" (
    echo  [OK] Ejecutable generado en:
    echo       %~dp0dist\ClearToolPC.exe
) else (
    echo  [ERROR] La compilacion fallo. Revisa los mensajes anteriores.
)

echo.
pause
