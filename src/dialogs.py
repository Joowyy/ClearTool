"""Diálogos modales de ClearTool PC."""

import subprocess
import tkinter as tk

from theme import (
    ACCENT, ACCENT_D, BG, BORDER, C_ERR, MUTED, SURFACE2, TEXT
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

    # ── Centrado ──────────────────────────────────────────────────────
    def _center(self, parent: tk.Tk):
        self.update_idletasks()
        px = parent.winfo_x() + (parent.winfo_width()  - self.winfo_width())  // 2
        py = parent.winfo_y() + (parent.winfo_height() - self.winfo_height()) // 2
        self.geometry(f"+{px}+{py}")

    # ── Pantalla inicial ──────────────────────────────────────────────
    def _build_initial(self):
        self._clear()

        tk.Label(
            self, text="Papelera de Reciclaje",
            bg=BG, fg=TEXT, font=("Segoe UI", 14, "bold"),
            padx=28, pady=20, anchor="w",
        ).pack(fill="x")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        tk.Label(
            self,
            text="¿Quieres revisar el contenido antes de vaciarla?",
            bg=BG, fg=TEXT, font=("Segoe UI", 10),
            padx=28, pady=14, anchor="w",
        ).pack(fill="x")

        tk.Label(
            self,
            text="Si eliges Revisar, se abrirá el Explorador.\n"
                 "Salva lo que necesites y pulsa Continuar.",
            bg=BG, fg=MUTED, font=("Segoe UI", 9),
            padx=28, justify="left",
        ).pack(anchor="w")

        tk.Frame(self, bg=BG, height=16).pack()
        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        row = tk.Frame(self, bg=BG, padx=28, pady=16)
        row.pack(fill="x")

        tk.Button(
            row, text="Omitir",
            bg=BG, fg=MUTED, font=("Segoe UI", 9),
            bd=0, cursor="hand2",
            activebackground=BG, activeforeground=TEXT,
            command=self._on_skip,
        ).pack(side="left")

        tk.Button(
            row, text="  Vaciar directamente  ",
            bg=SURFACE2, fg=C_ERR, font=("Segoe UI", 9, "bold"),
            bd=0, padx=12, pady=7, cursor="hand2",
            activebackground="#2a2a2a", activeforeground=C_ERR,
            command=self._on_delete,
        ).pack(side="right", padx=(8, 0))

        tk.Button(
            row, text="  Revisar primero  ",
            bg=ACCENT, fg="#000000", font=("Segoe UI", 9, "bold"),
            bd=0, padx=12, pady=7, cursor="hand2",
            activebackground=ACCENT_D, activeforeground="#000000",
            command=self._on_review,
        ).pack(side="right")

    # ── Pantalla de revisión (tras abrir Explorer) ────────────────────
    def _build_review(self):
        self._clear()

        tk.Label(
            self, text="Revisa la Papelera",
            bg=BG, fg=TEXT, font=("Segoe UI", 14, "bold"),
            padx=28, pady=20, anchor="w",
        ).pack(fill="x")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        tk.Label(
            self,
            text="El Explorador de Windows está abierto.\n"
                 "Cuando termines de salvar lo que necesites,\n"
                 "pulsa Continuar para vaciar el resto.",
            bg=BG, fg=MUTED, font=("Segoe UI", 9),
            padx=28, pady=18, justify="left",
        ).pack(anchor="w")

        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        row = tk.Frame(self, bg=BG, padx=28, pady=16)
        row.pack(fill="x")

        tk.Button(
            row, text="Cancelar",
            bg=BG, fg=MUTED, font=("Segoe UI", 9),
            bd=0, cursor="hand2",
            activebackground=BG, activeforeground=TEXT,
            command=self._on_skip,
        ).pack(side="left")

        tk.Button(
            row, text="  Continuar y vaciar  ",
            bg=ACCENT, fg="#000000", font=("Segoe UI", 9, "bold"),
            bd=0, padx=12, pady=7, cursor="hand2",
            activebackground=ACCENT_D, activeforeground="#000000",
            command=self._on_delete,
        ).pack(side="right")

        self.update_idletasks()
        self._center(self.master)

    # ── Helpers ───────────────────────────────────────────────────────
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
