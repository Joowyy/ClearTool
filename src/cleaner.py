"""Motor de limpieza: funciones que borran archivos del sistema."""

import ctypes
import shutil
import subprocess
from pathlib import Path


def _take_ownership(path: str) -> None:
    subprocess.run(
        ["takeown", "/f", path, "/r", "/d", "y"],
        capture_output=True, timeout=20,
    )
    subprocess.run(
        ["icacls", path, "/grant", "Administrators:F", "/t", "/q"],
        capture_output=True, timeout=20,
    )


def _remove_item(item: Path) -> bool:
    """Intenta borrar un item; si falla por permisos, toma ownership y reintenta."""
    try:
        shutil.rmtree(item) if item.is_dir() else item.unlink()
        return True
    except PermissionError:
        try:
            _take_ownership(str(item))
            shutil.rmtree(item) if item.is_dir() else item.unlink()
            return True
        except Exception:
            return False
    except Exception:
        return False


def delete_folder_contents(path: str) -> tuple[int, int]:
    """
    Borra el contenido de una carpeta (no la carpeta en sí).
    Devuelve (eliminados, bloqueados). -1 eliminados = ruta no existe.
    """
    p = Path(path)
    if not p.exists():
        return -1, 0

    deleted = locked = 0
    for item in list(p.iterdir()):
        if _remove_item(item):
            deleted += 1
        else:
            locked += 1
    return deleted, locked


def delete_thumbcache(path: str) -> tuple[int, int]:
    """Borra únicamente los archivos thumbcache_*.db e iconcache_*.db."""
    p = Path(path)
    if not p.exists():
        return -1, 0

    deleted = locked = 0
    for pattern in ("thumbcache_*.db", "iconcache_*.db"):
        for f in p.glob(pattern):
            if _remove_item(f):
                deleted += 1
            else:
                locked += 1
    return deleted, locked


def empty_recycle_bin() -> bool:
    """Vacía la Papelera de Reciclaje sin confirmación ni sonido."""
    SHERB_NO_CONFIRM    = 0x00000001
    SHERB_NO_PROGRESSUI = 0x00000002
    SHERB_NOSOUND       = 0x00000004
    flags = SHERB_NO_CONFIRM | SHERB_NO_PROGRESSUI | SHERB_NOSOUND
    result = ctypes.windll.shell32.SHEmptyRecycleBinW(None, None, flags)
    return result == 0


def clear_event_logs() -> tuple[int, int]:
    """Borra todos los registros del Visor de Eventos con wevtutil."""
    res = subprocess.run(["wevtutil", "el"], capture_output=True, text=True)
    cleared = failed = 0
    for name in res.stdout.strip().splitlines():
        name = name.strip()
        if not name:
            continue
        r = subprocess.run(["wevtutil", "cl", name], capture_output=True)
        if r.returncode == 0:
            cleared += 1
        else:
            failed += 1
    return cleared, failed


def run_command(cmd: list[str]) -> bool:
    """Ejecuta un comando externo y devuelve True si tuvo éxito."""
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=90)
        return r.returncode == 0
    except Exception:
        return False
