"""Tema visual — Midnight Dashboard."""

from tkinter import font as tkfont

# ── Paleta ────────────────────────────────────────────────────────────────────
BG       = "#060912"   # fondo principal — casi negro azulado
SURFACE  = "#0c1220"   # superficies secundarias
SURFACE2 = "#111b2e"   # fondo de tarjetas
SURFACE3 = "#192640"   # tarjeta en hover
BORDER   = "#1e2d47"   # bordes sutiles
ACCENT   = "#7c3aed"   # violeta principal
ACCENT_L = "#a78bfa"   # violeta claro (hover, resaltes)
ACCENT_D = "#5b21b6"   # violeta oscuro (pressed)
TEXT     = "#e2e8f0"   # texto principal
MUTED    = "#3d4f68"   # texto apagado oscuro
MUTED2   = "#64748b"   # texto apagado claro
LOG_BG   = "#030710"   # fondo del terminal
C_OK     = "#10b981"   # esmeralda — éxito
C_WARN   = "#f59e0b"   # ámbar — advertencia
C_ERR    = "#ef4444"   # rojo — error
C_INFO   = "#60a5fa"   # azul — información


def mono_font() -> str:
    families = tkfont.families()
    for candidate in ("Cascadia Code", "Cascadia Mono", "Consolas", "Courier New"):
        if candidate in families:
            return candidate
    return "Courier New"
