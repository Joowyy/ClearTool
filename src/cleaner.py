"""Motor de limpieza — orquesta los .bat de src/scripts/ y mide espacio liberado."""

import ctypes
import subprocess
import sys
from pathlib import Path


# ── Resolución robusta de SCRIPTS_DIR ────────────────────────────────────────
def _resolve_scripts_dir() -> Path:
    """
    Devuelve la ruta a src/scripts/ sea cual sea la forma de lanzar la app:
      · Exe compilado (PyInstaller --onefile):
            __file__ apunta a _MEIPASS (temp). Usamos sys.executable para
            encontrar el directorio real del .exe y bajamos a src/scripts/.
      · Script Python directo (python src/main.py):
            __file__ de cleaner.py es src/cleaner.py → parent = src/ → scripts/.
    """
    if getattr(sys, "frozen", False):
        # Ejecutable compilado: exe está en la raíz del proyecto
        return Path(sys.executable).parent / "src" / "scripts"
    return Path(__file__).parent / "scripts"


SCRIPTS_DIR      = _resolve_scripts_dir()
CREATE_NO_WINDOW = 0x08000000   # evita que aparezca ventana cmd al llamar bats


# ── Formato de tamaño ─────────────────────────────────────────────────────────
def format_size(b: int) -> str:
    """Convierte bytes a string legible: B, KB, MB, GB, TB."""
    if b <= 0:
        return "0 B"
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if b < 1024 or unit == "TB":
            return f"{b} {unit}" if unit == "B" else f"{b:.2f} {unit}"
        b /= 1024


# ── Medición de disco ─────────────────────────────────────────────────────────
def _get_size(path: Path) -> int:
    """Tamaño total de un archivo o árbol de directorios en bytes."""
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


def measure_paths(paths: list[str]) -> int:
    """Suma el tamaño actual de todas las rutas dadas."""
    return sum(_get_size(Path(p)) for p in paths)


# ── Ejecución de scripts ──────────────────────────────────────────────────────
def run_bat(bat_name: str) -> tuple[bool, str]:
    """
    Ejecuta un .bat de src/scripts/ en silencio (sin ventana cmd).
    Pasa /nopause para que el bat no espere tecla al final.
    Devuelve (éxito, mensaje_error).
    """
    bat_path = SCRIPTS_DIR / bat_name
    if not bat_path.exists():
        return False, f"No encontrado en: {bat_path}"
    try:
        result = subprocess.run(
            ["cmd", "/c", str(bat_path), "/nopause"],
            capture_output=True,
            creationflags=CREATE_NO_WINDOW,
            timeout=120,
        )
        if result.returncode != 0:
            return False, f"Código de salida: {result.returncode}"
        return True, ""
    except subprocess.TimeoutExpired:
        return False, "Timeout superado (>2 min)"
    except Exception as e:
        return False, str(e)


def run_bat_measured(bat_name: str, paths: list[str]) -> tuple[bool, int, str]:
    """
    Mide el tamaño de `paths` antes y después de ejecutar el bat.
    Devuelve (éxito, bytes_liberados, mensaje_error).
    """
    before       = measure_paths(paths)
    ok, err      = run_bat(bat_name)
    after        = measure_paths(paths)
    freed        = max(before - after, 0)
    return ok, freed, err


# ── Papelera de Reciclaje ─────────────────────────────────────────────────────
def get_recycle_bin_size() -> int:
    """Tamaño total de la Papelera de Reciclaje via API nativa de Windows."""
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
    Devuelve (éxito, bytes_liberados).
    """
    freed = get_recycle_bin_size()
    SHERB_NO_CONFIRM    = 0x00000001
    SHERB_NO_PROGRESSUI = 0x00000002
    SHERB_NOSOUND       = 0x00000004
    flags = SHERB_NO_CONFIRM | SHERB_NO_PROGRESSUI | SHERB_NOSOUND
    ok = ctypes.windll.shell32.SHEmptyRecycleBinW(None, None, flags) == 0
    return ok, (freed if ok else 0)


# ── Visor de Eventos ──────────────────────────────────────────────────────────
def clear_event_logs() -> tuple[int, int, int]:
    """
    Limpia todos los registros del Visor de Eventos con wevtutil.
    Devuelve (limpiados, protegidos, bytes_liberados).
    Mide el directorio winevt/Logs antes y después para calcular el espacio.
    """
    log_dir     = Path(r"C:\Windows\System32\winevt\Logs")
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
    freed      = max(size_before - size_after, 0)
    return cleared, failed, freed
