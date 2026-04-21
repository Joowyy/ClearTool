"""Diálogos modales de ClearTool PC — Midnight Dashboard."""

import subprocess
import tkinter as tk

from theme import (
    ACCENT, ACCENT_D, ACCENT_L,
    BG, BORDER,
    C_ERR,
    MUTED2, SURFACE2, SURFACE3,
    TEXT,
)


class RecycleBinDialog(tk.Toplevel):
    """
    Modal para la Papelera de Reciclaje.
    Uso correcto (hilo seguro):

        def _show():
            dlg = RecycleBinDialog(parent)
            parent.wait_window(dlg)        # ← procesa eventos de tkinter
            result["val"] = dlg.result
            done.set()
        parent.after(0, _show)
        done.wait()                        # ← bloquea sólo el hilo de limpieza
    """

    def __init__(self, parent: tk.Tk):
        super().__init__(parent)
        self.result: str = "skip"          # "delete" | "skip"

        self.title("Papelera de Reciclaje")
        self.configure(bg=BG)
        self.resizable(False, False)
        self.transient(parent)
        self.grab_set()
        self.protocol("WM_DELETE_WINDOW", self._on_skip)

        self._build_initial()
        self._center(parent)

    # ── Centrado ──────────────────────────────────────────────────────────────
    def _center(self, parent: tk.Tk):
        self.update_idletasks()
        px = parent.winfo_x() + (parent.winfo_width()  - self.winfo_width())  // 2
        py = parent.winfo_y() + (parent.winfo_height() - self.winfo_height()) // 2
        self.geometry(f"+{px}+{py}")

    # ── Pantalla inicial ──────────────────────────────────────────────────────
    def _build_initial(self):
        self._clear()

        # Barra de acento violeta superior
        tk.Frame(self, bg=ACCENT, height=3).pack(fill="x")

        # Encabezado
        hdr = tk.Frame(self, bg=BG, padx=28, pady=20)
        hdr.pack(fill="x")
        tk.Label(
            hdr, text="🗑  Papelera de Reciclaje",
            bg=BG, fg=TEXT, font=("Segoe UI", 13, "bold"),
            anchor="w",
        ).pack(fill="x")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        # Cuerpo
        body = tk.Frame(self, bg=SURFACE2, padx=28, pady=20)
        body.pack(fill="x")

        tk.Label(
            body,
            text="¿Quieres revisar el contenido antes de vaciarla?",
            bg=SURFACE2, fg=TEXT, font=("Segoe UI", 10, "bold"),
            anchor="w",
        ).pack(fill="x")

        tk.Frame(body, bg=SURFACE2, height=8).pack()

        tk.Label(
            body,
            text="Elige  Revisar  para abrir el Explorador.\n"
                 "Salva lo que necesites y pulsa  Continuar  cuando termines.",
            bg=SURFACE2, fg=MUTED2, font=("Segoe UI", 9),
            justify="left", anchor="w",
        ).pack(fill="x")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        # Fila de botones
        row = tk.Frame(self, bg=BG, padx=24, pady=16)
        row.pack(fill="x")

        # Omitir — texto sutil, izquierda
        lbl_skip = tk.Label(
            row, text="Omitir",
            bg=BG, fg=MUTED2, font=("Segoe UI", 9),
            cursor="hand2", padx=4,
        )
        lbl_skip.pack(side="left")
        lbl_skip.bind("<Button-1>", lambda _e: self._on_skip())
        lbl_skip.bind("<Enter>",    lambda _e: lbl_skip.config(fg=TEXT))
        lbl_skip.bind("<Leave>",    lambda _e: lbl_skip.config(fg=MUTED2))

        # Vaciar directamente — borde rojo + hover relleno
        frame_del = tk.Frame(row, bg=C_ERR, padx=1, pady=1)
        frame_del.pack(side="right", padx=(8, 0))
        inner_del = tk.Label(
            frame_del, text="  Vaciar directamente  ",
            bg=SURFACE3, fg=C_ERR,
            font=("Segoe UI", 9, "bold"),
            padx=10, pady=6, cursor="hand2",
        )
        inner_del.pack()
        inner_del.bind("<Button-1>", lambda _e: self._on_delete())
        inner_del.bind("<Enter>",    lambda _e: inner_del.config(bg=C_ERR, fg="#ffffff"))
        inner_del.bind("<Leave>",    lambda _e: inner_del.config(bg=SURFACE3, fg=C_ERR))

        # Revisar primero — violeta principal
        frame_rev = tk.Frame(row, bg=ACCENT_D, padx=1, pady=1)
        frame_rev.pack(side="right")
        inner_rev = tk.Label(
            frame_rev, text="  Revisar primero  ",
            bg=ACCENT, fg="#ffffff",
            font=("Segoe UI", 9, "bold"),
            padx=10, pady=6, cursor="hand2",
        )
        inner_rev.pack()
        inner_rev.bind("<Button-1>", lambda _e: self._on_review())
        inner_rev.bind("<Enter>",    lambda _e: inner_rev.config(bg=ACCENT_L, fg=BG))
        inner_rev.bind("<Leave>",    lambda _e: inner_rev.config(bg=ACCENT, fg="#ffffff"))

    # ── Pantalla de revisión (tras abrir Explorer) ───────────────────────────
    def _build_review(self):
        self._clear()

        # Barra de acento violeta superior
        tk.Frame(self, bg=ACCENT, height=3).pack(fill="x")

        # Encabezado
        hdr = tk.Frame(self, bg=BG, padx=28, pady=20)
        hdr.pack(fill="x")
        tk.Label(
            hdr, text="📂  Revisando la Papelera",
            bg=BG, fg=TEXT, font=("Segoe UI", 13, "bold"),
            anchor="w",
        ).pack(fill="x")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        # Cuerpo
        body = tk.Frame(self, bg=SURFACE2, padx=28, pady=20)
        body.pack(fill="x")

        tk.Label(
            body,
            text="El Explorador de Windows está abierto.",
            bg=SURFACE2, fg=TEXT, font=("Segoe UI", 10, "bold"),
            anchor="w",
        ).pack(fill="x")

        tk.Frame(body, bg=SURFACE2, height=8).pack()

        tk.Label(
            body,
            text="Salva lo que necesites y pulsa  Continuar\n"
                 "para vaciar el resto de la Papelera.",
            bg=SURFACE2, fg=MUTED2, font=("Segoe UI", 9),
            justify="left", anchor="w",
        ).pack(fill="x")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        # Fila de botones
        row = tk.Frame(self, bg=BG, padx=24, pady=16)
        row.pack(fill="x")

        # Cancelar — texto sutil, izquierda
        lbl_cancel = tk.Label(
            row, text="Cancelar",
            bg=BG, fg=MUTED2, font=("Segoe UI", 9),
            cursor="hand2", padx=4,
        )
        lbl_cancel.pack(side="left")
        lbl_cancel.bind("<Button-1>", lambda _e: self._on_skip())
        lbl_cancel.bind("<Enter>",    lambda _e: lbl_cancel.config(fg=TEXT))
        lbl_cancel.bind("<Leave>",    lambda _e: lbl_cancel.config(fg=MUTED2))

        # Continuar y vaciar — violeta principal
        frame_cont = tk.Frame(row, bg=ACCENT_D, padx=1, pady=1)
        frame_cont.pack(side="right")
        inner_cont = tk.Label(
            frame_cont, text="  Continuar y vaciar  ",
            bg=ACCENT, fg="#ffffff",
            font=("Segoe UI", 9, "bold"),
            padx=10, pady=6, cursor="hand2",
        )
        inner_cont.pack()
        inner_cont.bind("<Button-1>", lambda _e: self._on_delete())
        inner_cont.bind("<Enter>",    lambda _e: inner_cont.config(bg=ACCENT_L, fg=BG))
        inner_cont.bind("<Leave>",    lambda _e: inner_cont.config(bg=ACCENT, fg="#ffffff"))

        self.update_idletasks()
        self._center(self.master)

    # ── Helpers ───────────────────────────────────────────────────────────────
    def _clear(self):
        for w in self.winfo_children():
            w.destroy()

    def _on_review(self):
        subprocess.Popen(["explorer.exe", "shell:RecycleBinFolder"])
        self._build_review()

    def _on_delete(self):
        self.result = "delete"
        self.destroy()

    def _on_skip(self):
        self.result = "skip"
        self.destroy()
