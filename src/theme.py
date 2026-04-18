"""Paleta de colores y fuentes de ClearTool PC."""

from tkinter import font as tkfont

BG       = "#0d0d0d"
SURFACE  = "#141414"
SURFACE2 = "#1e1e1e"
BORDER   = "#252525"
ACCENT   = "#4ade80"
ACCENT_D = "#16a34a"
TEXT     = "#f4f4f5"
MUTED    = "#52525b"
LOG_BG   = "#0a0a0a"
C_OK     = "#4ade80"
C_WARN   = "#fbbf24"
C_ERR    = "#f87171"
C_INFO   = "#60a5fa"


def mono_font() -> str:
    families = tkfont.families()
    for candidate in ("Cascadia Code", "Cascadia Mono", "Consolas", "Courier New"):
        if candidate in families:
            return candidate
    return "Courier New"
