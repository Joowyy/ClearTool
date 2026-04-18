@echo off
:: IMPORTANTE: Ejecutar SIN admin. PyInstaller lo prohíbe expresamente.
:: El .exe generado pedirá UAC solo por sí mismo (flag --uac-admin).

:: Moverse siempre al directorio del propio .bat, sea cual sea el CWD
cd /d "%~dp0"

title ClearTool PC — Build
cls
echo.
echo  ================================================
echo    ClearTool PC  —  Compilar ejecutable
echo  ================================================
echo.

:: Verificar Python
python --version >nul 2>&1
if %errorLevel% neq 0 (
    echo  [ERROR] Python no encontrado.
    echo  Descargalo desde https://python.org
    pause & exit /b
)

:: Instalar / actualizar PyInstaller
echo  [1/3] Verificando PyInstaller...
pip install --upgrade pyinstaller >nul 2>&1
echo        OK.
echo.

:: Limpiar residuos de builds anteriores
echo  [2/3] Limpiando builds anteriores...
if exist "ClearToolPC.exe" del /f /q "ClearToolPC.exe"
if exist "_build_tmp"      rd /s /q "_build_tmp"
echo        OK.
echo.

:: Compilar — todo en una línea para evitar problemas con ^ en algunos sistemas
echo  [3/3] Compilando... (puede tardar 1-2 minutos)
echo.

pyinstaller --onefile --windowed --uac-admin --name "ClearToolPC" --distpath "." --workpath "_build_tmp" --specpath "_build_tmp" --paths "src" "src\main.py"

:: Limpiar temporales de compilación
if exist "_build_tmp" rd /s /q "_build_tmp"

echo.
echo  ------------------------------------------------
if exist "ClearToolPC.exe" (
    echo  [OK] Ejecutable listo:
    echo       %~dp0ClearToolPC.exe
    echo.
    echo  Cierra esta ventana y ejecuta ClearToolPC.exe
) else (
    echo  [ERROR] La compilacion fallo. Revisa los mensajes anteriores.
)
echo  ------------------------------------------------
echo.
pause
