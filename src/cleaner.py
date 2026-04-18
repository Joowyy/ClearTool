"""Motor de limpieza con seguimiento de espacio liberado en disco."""

import ctypes
import shutil
import subprocess
from pathlib import Path


# ── Utilidades de formato ─────────────────────────────────────────────────────

def format_size(b: int) -> str:
    """Convierte bytes a string legible: B, KB, MB, GB, TB."""
    if b <= 0:
        return "0 B"
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if b < 1024 or unit == "TB":
            return f"{b} {unit}" if unit == "B" else f"{b:.2f} {unit}"
        b /= 1024


# ── Helpers internos ──────────────────────────────────────────────────────────

def _get_size(path: Path) -> int:
    """Tamaño total de un archivo o árbol de directorio en bytes."""
    try:
        if path.is_file():
            return path.stat().st_size
        total = 0
        for f in path.rglob("*"):
            if f.is_file():
                try:
                    total += f.stat().st_size
                except Exception:
                    pass
        return total
    except Exception:
        return 0


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
    """Borra un archivo o carpeta. Si falla por permisos, toma ownership y reintenta."""
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


# ── Funciones públicas ────────────────────────────────────────────────────────

def delete_folder_contents(path: str) -> tuple[int, int, int]:
    """
    Borra el contenido de una carpeta (no la carpeta en sí).
    Retorna: (eliminados, bloqueados, bytes_liberados)
    eliminados == -1 → la ruta no existe.
    """
    p = Path(path)
    if not p.exists():
        return -1, 0, 0

    deleted = locked = freed = 0
    for item in list(p.iterdir()):
        size = _get_size(item)
        if _remove_item(item):
            deleted += 1
            freed += size
        else:
            locked += 1
    return deleted, locked, freed


def delete_thumbcache(path: str) -> tuple[int, int, int]:
    """
    Borra thumbcache_*.db e iconcache_*.db.
    Retorna: (eliminados, bloqueados, bytes_liberados)
    """
    p = Path(path)
    if not p.exists():
        return -1, 0, 0

    deleted = locked = freed = 0
    for pattern in ("thumbcache_*.db", "iconcache_*.db"):
        for f in p.glob(pattern):
            size = _get_size(f)
            if _remove_item(f):
                deleted += 1
                freed += size
            else:
                locked += 1
    return deleted, locked, freed


def get_recycle_bin_size() -> int:
    """
    Consulta el tamaño total de la Papelera de Reciclaje
    usando la API nativa de Windows (SHQueryRecycleBinW).
    """
    class SHQUERYRBINFO(ctypes.Structure):
        _fields_ = [
            ("cbSize",      ctypes.c_ulong),
            ("i64Size",     ctypes.c_longlong),
            ("i64NumItems", ctypes.c_longlong),
        ]

    info = SHQUERYRBINFO()
    info.cbSize = ctypes.sizeof(SHQUERYRBINFO)
    ret = ctypes.windll.shell32.SHQueryRecycleBinW(None, ctypes.byref(info))
    return info.i64Size if ret == 0 else 0


def empty_recycle_bin() -> tuple[bool, int]:
    """
    Vacía la Papelera de Reciclaje.
    Retorna: (éxito, bytes_liberados)
    Mide el tamaño ANTES de vaciar para informar el espacio recuperado.
    """
    freed = get_recycle_bin_size()
    SHERB_NO_CONFIRM    = 0x00000001
    SHERB_NO_PROGRESSUI = 0x00000002
    SHERB_NOSOUND       = 0x00000004
    flags = SHERB_NO_CONFIRM | SHERB_NO_PROGRESSUI | SHERB_NOSOUND
    ok = ctypes.windll.shell32.SHEmptyRecycleBinW(None, None, flags) == 0
    return ok, (freed if ok else 0)


def clear_event_logs() -> tuple[int, int, int]:
    """
    Borra todos los registros del Visor de Eventos con wevtutil.
    Retorna: (limpiados, protegidos, bytes_liberados)
    Mide el tamaño del directorio de logs antes y después.
    """
    log_dir = Path(r"C:\Windows\System32\winevt\Logs")
    size_before = _get_size(log_dir) if log_dir.exists() else 0

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

    size_after = _get_size(log_dir) if log_dir.exists() else 0
    freed = max(size_before - size_after, 0)
    return cleared, failed, freed


def run_command(cmd: list[str]) -> bool:
    """Ejecuta un comando externo. Retorna True si tuvo éxito."""
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=90)
        return r.returncode == 0
    except Exception:
        return False
