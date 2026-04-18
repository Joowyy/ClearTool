"""
Definición de los elementos de limpieza disponibles.

Cada categoría tiene:
  id       — identificador único
  label    — nombre visible en la UI
  desc     — descripción corta
  default  — activado por defecto (bool)
  type     — "bat_folder" | "bat_command" | "recycle_bin" | "event_logs"
  bat      — nombre del .bat en src/scripts/ (si aplica)
  paths    — rutas usadas para medir espacio antes/después (si aplica)
"""

import os


def build_categories() -> list[dict]:
    user      = os.environ.get("USERPROFILE", "")
    temp      = os.environ.get("TEMP", "")
    local_app = os.environ.get("LOCALAPPDATA", "")

    return [
        {
            "id":      "temp_user",
            "label":   "Archivos Temporales de Usuario",
            "desc":    "%TEMP% — temporales del usuario actual",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaTempUsuario.bat",
            "paths":   [temp],
        },
        {
            "id":      "temp_win",
            "label":   "Windows Temp",
            "desc":    "Temporales del sistema (C:\\Windows\\Temp)",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaWindowsTemp.bat",
            "paths":   [r"C:\Windows\Temp"],
        },
        {
            "id":      "prefetch",
            "label":   "Prefetch",
            "desc":    "Caché de arranque de aplicaciones",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaPrefetch.bat",
            "paths":   [r"C:\Windows\Prefetch"],
        },
        {
            "id":      "install_temp",
            "label":   "Temporales de Instalación de Windows",
            "desc":    "Residuos de upgrades del SO ($WINDOWS.~BT, ~WS)",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaInstalacionWindows.bat",
            "paths":   [r"C:\$WINDOWS.~BT", r"C:\$WINDOWS.~WS"],
        },
        {
            "id":      "delivery_opt",
            "label":   "Optimización de Distribución",
            "desc":    "Caché P2P de Windows Update (puede ser varios GB)",
            "default": True,
            "type":    "bat_command",
            "bat":     "LimpiezaOptimizacionDistribucion.bat",
        },
        {
            "id":      "thumbnails",
            "label":   "Miniaturas",
            "desc":    "Caché de vistas previas de imágenes y carpetas",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaMiniaturas.bat",
            "paths":   [os.path.join(local_app, r"Microsoft\Windows\Explorer")],
        },
        {
            "id":      "defender",
            "label":   "Microsoft Defender (no críticos)",
            "desc":    "Historial de análisis no esenciales del antivirus",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaDefender.bat",
            "paths":   [
                r"C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory",
            ],
        },
        {
            "id":      "wer",
            "label":   "Informes de Errores de Windows",
            "desc":    "Diagnósticos y reportes de fallos del sistema",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaInformesErrores.bat",
            "paths":   [
                os.path.join(local_app, r"Microsoft\Windows\WER"),
                r"C:\ProgramData\Microsoft\Windows\WER",
            ],
        },
        {
            "id":      "win_update_logs",
            "label":   "Registros de Actualización de Windows",
            "desc":    "Logs de sesión de Windows Update — desactivado por defecto",
            "default": False,
            "type":    "bat_folder",
            "bat":     "LimpiezaRegistrosActualizacion.bat",
            "paths":   [r"C:\Windows\Logs\WindowsUpdate"],
        },
        {
            "id":      "dx_cache",
            "label":   "Caché de Sombreador DirectX",
            "desc":    "Shaders compilados, se regeneran automáticamente",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaCacheDirectX.bat",
            "paths":   [os.path.join(local_app, "D3DSCache")],
        },
        {
            "id":      "inet_cache",
            "label":   "Temporales de Internet",
            "desc":    "Caché de Microsoft Edge / Internet Explorer",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaTemporalesInternet.bat",
            "paths":   [os.path.join(local_app, r"Microsoft\Windows\INetCache")],
        },
        {
            "id":      "store_cache",
            "label":   "Caché de Microsoft Store",
            "desc":    "Limpia temporales de la tienda sin abrirla",
            "default": True,
            "type":    "bat_folder",
            "bat":     "LimpiezaStoreCache.bat",
            "paths":   [
                os.path.join(local_app, r"Packages\Microsoft.WindowsStore_8wekyb3d8bbwe\LocalCache"),
                os.path.join(local_app, r"Packages\Microsoft.WindowsStore_8wekyb3d8bbwe\AC\INetCache"),
                os.path.join(local_app, r"Packages\Microsoft.WindowsStore_8wekyb3d8bbwe\AC\Temp"),
            ],
        },
        {
            "id":      "dns_cache",
            "label":   "Caché DNS",
            "desc":    "Vacía la caché DNS — resuelve problemas de conexión",
            "default": True,
            "type":    "bat_command",
            "bat":     "LimpiezaDNS.bat",
        },
        {
            "id":      "recycle_bin",
            "label":   "Papelera de Reciclaje",
            "desc":    "Te preguntará si revisar el contenido antes de vaciarla",
            "default": True,
            "type":    "recycle_bin",
        },
        {
            "id":      "event_logs",
            "label":   "Registros del Visor de Eventos",
            "desc":    "Limpia todos los logs del Event Viewer",
            "default": True,
            "type":    "event_logs",
        },
    ]
