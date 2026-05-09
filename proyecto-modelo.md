# ClearTool PC — Modelo del Proyecto

Suite de optimización para Windows que limpia cachés del sistema, vacía la papelera, borra logs del Visor de Eventos y permite desinstalar apps preinstaladas (debloat). Interfaz en Tkinter con tema "Midnight Dashboard" y elevación automática a administrador.

---

## Directorios y cachés que se borran

| Categoría | Ruta / Acción | Por defecto |
|---|---|---|
| Archivos Temporales de Usuario | `%TEMP%` | Sí |
| Windows Temp | `C:\Windows\Temp` | Sí |
| Prefetch | `C:\Windows\Prefetch` | Sí |
| Temporales de Instalación de Windows | `C:\$WINDOWS.~BT`, `C:\$WINDOWS.~WS` | Sí |
| Optimización de Distribución | Caché P2P de Windows Update (comando) | Sí |
| Miniaturas | `%LOCALAPPDATA%\Microsoft\Windows\Explorer` | Sí |
| Microsoft Defender (no críticos) | `C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory` | Sí |
| Informes de Errores de Windows (WER) | `%LOCALAPPDATA%\Microsoft\Windows\WER` y `C:\ProgramData\Microsoft\Windows\WER` | Sí |
| Registros de Actualización de Windows | `C:\Windows\Logs\WindowsUpdate` | No |
| Caché de Sombreador DirectX | `%LOCALAPPDATA%\D3DSCache` | Sí |
| Temporales de Internet (Edge / IE) | `%LOCALAPPDATA%\Microsoft\Windows\INetCache` | Sí |
| Caché de Microsoft Store | `%LOCALAPPDATA%\Packages\Microsoft.WindowsStore_8wekyb3d8bbwe\` (`LocalCache`, `AC\INetCache`, `AC\Temp`) | Sí |
| Caché DNS | `ipconfig /flushdns` (comando) | Sí |
| Papelera de Reciclaje | Vaciado vía API de Shell | Sí |
| Registros del Visor de Eventos | Borrado de todos los logs del Event Viewer | Sí |

---

## Funcionalidades importantes

- **Elevación automática a administrador** (`main.py`): si el proceso no tiene privilegios, se relanza con UAC mediante `ShellExecuteW("runas", ...)`.
- **Limpieza por categorías** (`categories.py` + `cleaner.py`): cada categoría se ejecuta como `bat_folder`, `bat_command`, `recycle_bin` o `event_logs`, midiendo el espacio antes/después para reportar lo liberado.
- **Motor de Debloat de apps Appx** (`debloat.py`): lista paquetes instalados vía PowerShell y permite desinstalarlos con clasificación por riesgo (`safe` / `moderate` / `system`) y categoría (Microsoft, OEM, etc.). Genera un snapshot en `~/.cleartool/debloat_snapshot.json` antes de actuar.
- **Dashboard del sistema** (`app.py`): muestra en tiempo real uso de RAM (`GlobalMemoryStatusEx`), uso de disco (`shutil.disk_usage`) y estado de red (test contra `8.8.8.8:53`).
- **Diálogo de Papelera** (`dialogs.py`): permite revisar el contenido antes de vaciarla.
- **Tema visual unificado** (`theme.py`): paleta "Midnight Dashboard" con acentos, colores semánticos (`C_OK`, `C_WARN`, `C_ERR`, `C_INFO`) y fuentes UI/mono.
- **Ejecución asíncrona**: las limpiezas corren en hilos (`threading`) para no bloquear la UI, con log en vivo de cada operación.
