"""Ventana principal de ClearTool PC."""

import datetime
import threading
import tkinter as tk
from tkinter import font as tkfont

from categories import build_categories
from cleaner import (
    clear_event_logs,
    delete_folder_contents,
    delete_thumbcache,
    empty_recycle_bin,
    run_command,
)
from dialogs import RecycleBinDialog
from theme import (
    ACCENT, ACCENT_D, BG, BORDER, C_ERR, C_INFO, C_OK, C_WARN,
    LOG_BG, MUTED, SURFACE, SURFACE2, TEXT, mono_font,
)


class App(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("ClearTool PC")
        self.configure(bg=BG)
        self.geometry("980x660")
        self.minsize(820, 560)

        self.categories = build_categories()
        self.vars: dict[str, tk.BooleanVar] = {}

        self._build()
        self._center()
        self._log("ClearTool PC listo.", C_INFO)
        self._log("Selecciona los elementos y pulsa  Iniciar limpieza.", MUTED)

    # ── Centrado ──────────────────────────────────────────────────────
    def _center(self):
        self.update_idletasks()
        sw, sh = self.winfo_screenwidth(), self.winfo_screenheight()
        w, h = self.winfo_width(), self.winfo_height()
        self.geometry(f"+{(sw - w) // 2}+{(sh - h) // 2}")

    # ── Construcción UI ───────────────────────────────────────────────
    def _build(self):
        self._header()
        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")
        self._body()
        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")
        self._footer()

    def _header(self):
        f = tk.Frame(self, bg=BG, padx=28, pady=18)
        f.pack(fill="x")
        tk.Label(f, text="ClearTool", bg=BG, fg=TEXT,
                 font=("Segoe UI", 20, "bold")).pack(side="left")
        tk.Label(f, text="PC", bg=BG, fg=ACCENT,
                 font=("Segoe UI", 20, "bold")).pack(side="left", padx=(4, 0))
        tk.Label(f, text="  Limpieza profesional del sistema",
                 bg=BG, fg=MUTED,
                 font=("Segoe UI", 9)).pack(side="left", pady=(9, 0))

    def _body(self):
        body = tk.Frame(self, bg=BG)
        body.pack(fill="both", expand=True)
        self._left_panel(body)
        tk.Frame(body, bg=BORDER, width=1).pack(side="left", fill="y")
        self._right_panel(body)

    # ── Panel izquierdo ───────────────────────────────────────────────
    def _left_panel(self, parent):
        left = tk.Frame(parent, bg=BG, width=340)
        left.pack(side="left", fill="y")
        left.pack_propagate(False)

        hdr = tk.Frame(left, bg=BG, padx=20, pady=12)
        hdr.pack(fill="x")
        tk.Label(hdr, text="ELEMENTOS A LIMPIAR", bg=BG, fg=MUTED,
                 font=("Segoe UI", 8, "bold")).pack(side="left")
        tk.Frame(left, bg=BORDER, height=1).pack(fill="x")

        outer = tk.Frame(left, bg=BG)
        outer.pack(fill="both", expand=True)

        canvas = tk.Canvas(outer, bg=BG, bd=0, highlightthickness=0)
        sb = tk.Scrollbar(outer, orient="vertical", command=canvas.yview, width=6)
        inner = tk.Frame(canvas, bg=BG)

        inner.bind("<Configure>",
                   lambda e: canvas.configure(scrollregion=canvas.bbox("all")))
        canvas.create_window((0, 0), window=inner, anchor="nw", width=320)
        canvas.configure(yscrollcommand=sb.set)
        canvas.pack(side="left", fill="both", expand=True)
        sb.pack(side="right", fill="y")
        canvas.bind_all("<MouseWheel>",
                        lambda e: canvas.yview_scroll(-1 * (e.delta // 120), "units"))

        for cat in self.categories:
            self._category_row(inner, cat)

    def _category_row(self, parent, cat: dict):
        var = tk.BooleanVar(value=cat["default"])
        self.vars[cat["id"]] = var

        row = tk.Frame(parent, bg=BG, padx=16, pady=8)
        row.pack(fill="x")

        top = tk.Frame(row, bg=BG)
        top.pack(fill="x")

        tk.Checkbutton(
            top, variable=var,
            bg=BG, fg=TEXT,
            activebackground=BG, activeforeground=TEXT,
            selectcolor=SURFACE2, bd=0,
            font=("Segoe UI", 10),
            text=cat["label"],
            anchor="w", cursor="hand2",
        ).pack(side="left", fill="x", expand=True)

        if not cat["default"]:
            tk.Label(top, text="OFF", bg=BG, fg=MUTED,
                     font=("Segoe UI", 7, "bold")).pack(side="right")

        tk.Label(
            row, text=cat["desc"],
            bg=BG, fg=MUTED, font=("Segoe UI", 8),
            wraplength=270, justify="left", anchor="w",
        ).pack(fill="x", padx=(22, 0))

        tk.Frame(row, bg=BORDER, height=1).pack(fill="x", pady=(8, 0))

    # ── Panel derecho — terminal ───────────────────────────────────────
    def _right_panel(self, parent):
        right = tk.Frame(parent, bg=SURFACE)
        right.pack(side="right", fill="both", expand=True)

        th = tk.Frame(right, bg=SURFACE, padx=16, pady=10)
        th.pack(fill="x")

        dot = tk.Frame(th, bg=SURFACE)
        dot.pack(side="left")
        for col in ("#ef4444", "#f59e0b", "#22c55e"):
            tk.Label(dot, text="●", bg=SURFACE, fg=col,
                     font=("Segoe UI", 10)).pack(side="left", padx=2)
        tk.Label(th, text="  Log de depuración", bg=SURFACE, fg=MUTED,
                 font=("Segoe UI", 9)).pack(side="left")
        tk.Button(
            th, text="Limpiar log",
            bg=SURFACE, fg=MUTED, font=("Segoe UI", 8),
            bd=0, cursor="hand2",
            activebackground=SURFACE, activeforeground=TEXT,
            command=self._clear_log,
        ).pack(side="right")

        tk.Frame(right, bg=BORDER, height=1).pack(fill="x")

        mono = mono_font()
        self.log_widget = tk.Text(
            right, bg=LOG_BG, fg=TEXT,
            font=(mono, 9),
            bd=0, padx=14, pady=12,
            state="disabled", wrap="word",
            insertbackground=TEXT, selectbackground="#2a2a2a",
        )
        self.log_widget.pack(fill="both", expand=True)

        for tag, fg in [
            (C_OK, C_OK), (C_WARN, C_WARN), (C_ERR, C_ERR),
            (C_INFO, C_INFO), (MUTED, MUTED),
        ]:
            self.log_widget.tag_config(tag, foreground=fg)
        self.log_widget.tag_config(
            "bold", font=(mono, 9, "bold"), foreground=TEXT
        )
        self.log_widget.tag_config("ts", foreground="#333333")

    # ── Footer ────────────────────────────────────────────────────────
    def _footer(self):
        f = tk.Frame(self, bg=BG, padx=28, pady=14)
        f.pack(fill="x")

        self.lbl_status = tk.Label(
            f, text="Listo.", bg=BG, fg=MUTED, font=("Segoe UI", 9)
        )
        self.lbl_status.pack(side="left")

        self.btn_run = tk.Button(
            f, text="  Iniciar limpieza  ",
            bg=ACCENT, fg="#000000", font=("Segoe UI", 10, "bold"),
            bd=0, padx=14, pady=7, cursor="hand2", relief="flat",
            activebackground=ACCENT_D, activeforeground="#000000",
            command=self._start,
        )
        self.btn_run.pack(side="right")

        tk.Button(
            f, text="Deseleccionar todo",
            bg=BG, fg=MUTED, font=("Segoe UI", 9),
            bd=0, cursor="hand2",
            activebackground=BG, activeforeground=TEXT,
            command=lambda: [v.set(False) for v in self.vars.values()],
        ).pack(side="right", padx=(0, 12))

        tk.Button(
            f, text="Seleccionar todo",
            bg=BG, fg=MUTED, font=("Segoe UI", 9),
            bd=0, cursor="hand2",
            activebackground=BG, activeforeground=TEXT,
            command=lambda: [v.set(True) for v in self.vars.values()],
        ).pack(side="right", padx=(0, 4))

    # ── Log ───────────────────────────────────────────────────────────
    def _log(self, msg: str, color: str = TEXT):
        def _do():
            ts = datetime.datetime.now().strftime("%H:%M:%S")
            w = self.log_widget
            w.configure(state="normal")
            w.insert("end", f"[{ts}] ", "ts")
            w.insert("end", f"{msg}\n", color)
            w.see("end")
            w.configure(state="disabled")
        self.after(0, _do)

    def _log_sep(self):
        self._log("─" * 46, MUTED)

    def _clear_log(self):
        self.log_widget.configure(state="normal")
        self.log_widget.delete("1.0", "end")
        self.log_widget.configure(state="disabled")

    # ── Diálogo papelera (hilo-seguro) ────────────────────────────────
    def _ask_recycle_bin(self) -> str:
        """
        Muestra el diálogo en el hilo principal usando wait_window(),
        que es el patrón correcto de tkinter para modales.
        El hilo de limpieza espera en done.wait() sin bloquear la UI.
        """
        result: dict[str, str] = {"val": "skip"}
        done = threading.Event()

        def _show():
            dlg = RecycleBinDialog(self)
            self.wait_window(dlg)      # ← procesa eventos de tkinter; no bloquea UI
            result["val"] = dlg.result
            done.set()

        self.after(0, _show)
        done.wait()                    # ← sólo bloquea el hilo de limpieza
        return result["val"]

    # ── Limpieza ──────────────────────────────────────────────────────
    def _start(self):
        selected = [c for c in self.categories if self.vars[c["id"]].get()]
        if not selected:
            self._log("No hay ningún elemento seleccionado.", C_WARN)
            return
        self._set_running(True)
        threading.Thread(target=self._run, args=(selected,), daemon=True).start()

    def _set_running(self, state: bool):
        self.btn_run.configure(
            state="disabled" if state else "normal",
            text="  Limpiando...  " if state else "  Iniciar limpieza  ",
            bg="#2a2a2a" if state else ACCENT,
            fg=MUTED if state else "#000000",
        )

    def _run(self, selected: list[dict]):
        self._log_sep()
        self._log(f"LIMPIEZA INICIADA — {len(selected)} elemento(s)", "bold")
        self._log_sep()

        total_del = total_lock = 0
        for i, cat in enumerate(selected, 1):
            self._log(f"[{i}/{len(selected)}]  {cat['label']}", "bold")
            d, l = self._process(cat)
            total_del  += max(d, 0)
            total_lock += l

        self._log_sep()
        self._log(
            f"COMPLETADO  ·  Eliminados: {total_del}  ·  Bloqueados: {total_lock}",
            C_OK if total_lock == 0 else C_WARN,
        )
        if total_lock:
            self._log(
                "Los archivos bloqueados están en uso. Se liberarán al reiniciar.", MUTED
            )
        self._log_sep()

        self.after(0, lambda: self._set_running(False))
        self.after(0, lambda: self.lbl_status.configure(
            text=f"Completado — {total_del} eliminados · {total_lock} bloqueados"
        ))

    def _process(self, cat: dict) -> tuple[int, int]:
        t = cat["type"]

        if t == "folder":
            total_d = total_l = 0
            for path in cat["paths"]:
                self._log(f"    → {path}", MUTED)
                d, l = delete_folder_contents(path)
                if d == -1:
                    self._log("      Ruta no encontrada, omitida.", MUTED)
                else:
                    self._log(
                        f"      ✓ {d} eliminados · {l} bloqueados",
                        C_OK if l == 0 else C_WARN,
                    )
                    total_d += d
                    total_l += l
            return total_d, total_l

        elif t == "thumbcache":
            total_d = total_l = 0
            for path in cat["paths"]:
                self._log(f"    → {path}", MUTED)
                d, l = delete_thumbcache(path)
                if d == -1:
                    self._log("      Ruta no encontrada, omitida.", MUTED)
                else:
                    self._log(f"      ✓ {d} archivos eliminados", C_OK)
                    total_d += d
                    total_l += l
            return total_d, total_l

        elif t == "recycle_bin":
            self._log("    → Esperando respuesta del usuario...", MUTED)
            action = self._ask_recycle_bin()
            if action == "delete":
                ok = empty_recycle_bin()
                self._log(
                    "      ✓ Papelera vaciada." if ok
                    else "      ! Error al vaciar la papelera.",
                    C_OK if ok else C_WARN,
                )
                return (1, 0) if ok else (0, 0)
            else:
                self._log("      — Omitido por el usuario.", MUTED)
                return 0, 0

        elif t == "event_logs":
            self._log("    → Limpiando registros de eventos...", MUTED)
            cleared, failed = clear_event_logs()
            self._log(
                f"      ✓ {cleared} registros limpiados · {failed} protegidos", C_OK
            )
            return cleared, failed

        elif t == "command":
            self._log(f"    → {' '.join(cat['cmd'][:2])} ...", MUTED)
            ok = run_command(cat["cmd"])
            self._log(
                "      ✓ Completado" if ok
                else "      ! No disponible en este sistema",
                C_OK if ok else C_WARN,
            )
            return (1, 0) if ok else (0, 0)

        return 0, 0
