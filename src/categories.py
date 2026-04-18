"""Definición de los elementos de limpieza disponibles."""

import os


def build_categories() -> list[dict]:
    user = os.environ.get("USERPROFILE", "")
    temp = os.environ.get("TEMP", "")

    return [
        {
            "id":      "temp_user",
            "label":   "Archivos Temporales de Usuario",
            "desc":    "%TEMP% — temporales del usuario actual",
            "default": True,
            "type":    "folder",
            "paths":   [temp],
        },
        {
            "id":      "temp_win",
            "label":   "Windows Temp",
            "desc":    "Temporales del sistema (C:\\Windows\\Temp)",
            "default": True,
            "type":    "folder",
            "paths":   [r"C:\Windows\Temp"],
        },
        {
            "id":      "prefetch",
            "label":   "Prefetch",
            "desc":    "Caché de arranque de aplicaciones",
            "default": True,
            "type":    "folder",
            "paths":   [r"C:\Windows\Prefetch"],
        },
        {
            "id":      "install_temp",
            "label":   "Temporales de Instalación de Windows",
            "desc":    "Residuos de upgrades del SO ($WINDOWS.~BT, ~WS)",
            "default": True,
            "type":    "folder",
            "paths":   [r"C:\$WINDOWS.~BT", r"C:\$WINDOWS.~WS"],
        },
        {
            "id":      "delivery_opt",
            "label":   "Optimización de Distribución",
            "desc":    "Caché P2P de Windows Update (puede ser varios GB)",
            "default": True,
            "type":    "command",
            "cmd":     [
                "powershell", "-Command",
                "Delete-DeliveryOptimizationCache -Force -ErrorAction SilentlyContinue",
            ],
        },
        {
            "id":      "thumbnails",
            "label":   "Miniaturas",
            "desc":    "Caché de vistas previas de imágenes y carpetas",
            "default": True,
            "type":    "thumbcache",
            "paths":   [os.path.join(user, r"AppData\Local\Microsoft\Windows\Explorer")],
        },
        {
            "id":      "defender",
            "label":   "Microsoft Defender (no críticos)",
            "desc":    "Historial de análisis no esenciales del antivirus",
            "default": True,
            "type":    "folder",
            "paths":   [
                r"C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory",
            ],
        },
        {
            "id":      "wer",
            "label":   "Informes de Errores de Windows",
            "desc":    "Diagnósticos y reportes de fallos del sistema",
            "default": True,
            "type":    "folder",
            "paths":   [
                os.path.join(user, r"AppData\Local\Microsoft\Windows\WER"),
                r"C:\ProgramData\Microsoft\Windows\WER",
            ],
        },
        {
            "id":      "win_update_logs",
            "label":   "Registros de Actualización de Windows",
            "desc":    "Logs de sesión de Windows Update — desactivado por defecto",
            "default": False,
            "type":    "folder",
            "paths":   [r"C:\Windows\Logs\WindowsUpdate"],
        },
        {
            "id":      "dx_cache",
            "label":   "Caché de Sombreador DirectX",
            "desc":    "Shaders compilados, se regeneran automáticamente",
            "default": True,
            "type":    "folder",
            "paths":   [os.path.join(user, r"AppData\Local\D3DSCache")],
        },
        {
            "id":      "inet_cache",
            "label":   "Temporales de Internet",
            "desc":    "Caché de Microsoft Edge / Internet Explorer",
            "default": True,
            "type":    "folder",
            "paths":   [os.path.join(user, r"AppData\Local\Microsoft\Windows\INetCache")],
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
