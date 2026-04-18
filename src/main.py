"""ClearTool PC — Entry point."""

import ctypes
import os
import sys

# Asegura que los módulos del mismo directorio se encuentren
# tanto al ejecutar directamente como desde el .exe compilado
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def is_admin() -> bool:
    try:
        return bool(ctypes.windll.shell32.IsUserAnAdmin())
    except Exception:
        return False


def elevate() -> None:
    ctypes.windll.shell32.ShellExecuteW(
        None, "runas", sys.executable,
        f'"{os.path.abspath(sys.argv[0])}"', None, 1,
    )
    sys.exit(0)


def main():
    if not is_admin():
        elevate()

    from app import App
    app = App()
    app.mainloop()


if __name__ == "__main__":
    main()
