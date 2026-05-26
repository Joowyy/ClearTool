# Script para compilar ClearTool como ejecutable .exe
# Uso: .\build-release.ps1

Write-Host "=== ClearTool Build Release ===" -ForegroundColor Cyan
Write-Host ""

# Verificar que estamos en el directorio correcto
if (-not (Test-Path "package.json")) {
    Write-Host "ERROR: package.json no encontrado. Ejecuta este script desde la raíz del proyecto." -ForegroundColor Red
    exit 1
}

Write-Host "1. Compilando frontend (TypeScript + Vite)..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Compilación del frontend falló" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "2. Compilando ejecutable Tauri (Rust + Tauri)..." -ForegroundColor Yellow
Write-Host "   Esto puede tardar 5-15 minutos la primera vez..." -ForegroundColor Gray
npm run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Compilación de Tauri falló" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Compilación completada ===" -ForegroundColor Green
Write-Host ""
Write-Host "El ejecutable se encuentra en:" -ForegroundColor Cyan
Write-Host "  src-tauri/target/release/bundle/msi/ClearTool_0.1.0_x64_en-US.msi" -ForegroundColor White
Write-Host "  src-tauri/target/release/ClearTool.exe" -ForegroundColor White
Write-Host ""
Write-Host "Puedes ejecutar directamente:" -ForegroundColor Cyan
Write-Host "  .\src-tauri\target\release\ClearTool.exe" -ForegroundColor White
