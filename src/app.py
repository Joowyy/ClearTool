"""ClearTool PC — Ventana principal. Tema Midnight Dashboard."""

import datetime
import threading
import tkinter as tk
from tkinter import font as tkfont

from categories import build_categories
from cleaner import (
    SCRIPTS_DIR,
    clear_event_logs,
    empty_recycle_bin,
    format_size,
    run_bat,
    run_bat_measured,
)
from dialogs import RecycleBinDialog
from theme import (
    ACCENT, ACCENT_D, ACCENT_L,
    BG, BORDER,
    C_ERR, C_INFO, C_OK, C_WARN,
    LOG_BG, MUTED, MUTED2,
    SURFACE, SURFACE2, SURFACE3,
    TEXT, mono_font,
)


# ── Toggle switch ─────────────────────────────────────────────────────────────
class ToggleSwitch(tk.Canvas):
    """Interruptor de palanca en forma de píldora dibujado con Canvas."""

    W, H = 44, 24

    def __init__(self, parent: tk.Widget, variable: tk.BooleanVar, bg: str, **kw):
        super().__init__(
            parent, width=self.W, height=self.H,
            bg=bg, bd=0, highlightthickness=0, cursor="hand2", **kw,
        )
        self._var = variable
        self._bg  = bg
        self.bind("<Button-1>", lambda _: self._var.set(not self._var.get()))
        self._var.trace_add("write", lambda *_: self._redraw())
        self._redraw()

    def set_bg(self, bg: str):
        self._bg = bg
        self.configure(bg=bg)
        self._redraw()

    def _redraw(self):
        self.delete("all")
        on    = self._var.get()
        track = ACCENT if on else MUTED
        r     = self.H // 2

        # Píldora (dos óvalos + rectángulo central)
        self.create_oval(0, 0, self.H, self.H, fill=track, outline="")
        self.create_oval(self.W - self.H, 0, self.W, self.H, fill=track, outline="")
        self.create_rectangle(r, 0, self.W - r, self.H, fill=track, outline="")

        # Círculo deslizante
        pad = 3
        cx  = (self.W - r) if on else r
        self.create_oval(
            cx - r + pad, pad,
            cx + r - pad, self.H - pad,
            fill="white", outline="",
        )


# ── Aplicación ────────────────────────────────────────────────────────────────
class App(tk.Tk):

    def __init__(self):
        super().__init__()
        self.title("ClearTool PC")
        self.configure(bg=BG)
        self.geometry("1020x680")
        self.minsize(860, 580)

        self.categories  = build_categories()
        self.vars: dict[str, tk.BooleanVar] = {}
        self.running     = False
        self._freed_total = 0
        self._anim_idx   = 0

        self._build()
        self._center()

        self._log("ClearTool PC listo.", C_INFO)
        self._log(f"Scripts: {SCRIPTS_DIR}", MUTED)
        if not SCRIPTS_DIR.exists():
            self._log("⚠  Directorio de scripts no encontrado.", C_WARN)
        self._log("Selecciona los elementos y pulsa  Iniciar limpieza.", MUTED2)

    # ── Centrado ──────────────────────────────────────────────────────
    def _center(self):
        self.update_idletasks()
        sw, sh = self.winfo_screenwidth(), self.winfo_screenheight()
        w,  h  = self.winfo_width(),       self.winfo_height()
        self.geometry(f"+{(sw - w) // 2}+{(sh - h) // 2}")

    # ── Construcción ──────────────────────────────────────────────────
    def _build(self):
        self._build_header()
        # Línea decorativa degradada simulada con dos frames
        tk.Frame(self, bg=ACCENT,  height=1).pack(fill="x")
        tk.Frame(self, bg=BORDER,  height=1).pack(fill="x")
        self._build_body()
        tk.Frame(self, bg=BORDER,  height=1).pack(fill="x")
        self._build_footer()

    # ── Header ────────────────────────────────────────────────────────
    def _build_header(self):
        hdr = tk.Frame(self, bg=SURFACE, padx=28, pady=0)
        hdr.pack(fill="x")

        # Lado izquierdo — logo + subtítulo
        left = tk.Frame(hdr, bg=SURFACE)
        left.pack(side="left", pady=20)

        title_row = tk.Frame(left, bg=SURFACE)
        title_row.pack(anchor="w")

        tk.Label(
            title_row, text="Clear", bg=SURFACE, fg=TEXT,
            font=("Segoe UI", 22, "bold"),
        ).pack(side="left")
        tk.Label(
            title_row, text="Tool", bg=SURFACE, fg=ACCENT,
            font=("Segoe UI", 22, "bold"),
        ).pack(side="left")
        tk.Label(
            title_row, text=" PC", bg=SURFACE, fg=MUTED2,
            font=("Segoe UI", 14),
        ).pack(side="left", pady=(6, 0))

        tk.Label(
            left, text="Sistema de limpieza avanzado para Windows",
            bg=SURFACE, fg=MUTED2, font=("Segoe UI", 9),
        ).pack(anchor="w")

        # Lado derecho — estado + espacio liberado
        right = tk.Frame(hdr, bg=SURFACE)
        right.pack(side="right", pady=20)

        # Badge de espacio liberado
        freed_box = tk.Frame(right, bg=SURFACE2, padx=14, pady=8)
        freed_box.pack(side="right", padx=(12, 0))

        tk.Label(
            freed_box, text="ESPACIO LIBERADO",
            bg=SURFACE2, fg=MUTED2, font=("Segoe UI", 7, "bold"),
        ).pack()
        self._lbl_freed = tk.Label(
            freed_box, text="0 B",
            bg=SURFACE2, fg=ACCENT_L,
            font=("Segoe UI", 16, "bold"),
        )
        self._lbl_freed.pack()

        # Indicador de estado
        status_box = tk.Frame(right, bg=SURFACE)
        status_box.pack(side="right")

        self._dot = tk.Label(
            status_box, text="●", bg=SURFACE, fg=C_OK,
            font=("Segoe UI", 11),
        )
        self._dot.pack(side="left")
        self._lbl_state = tk.Label(
            status_box, text="Listo",
            bg=SURFACE, fg=MUTED2, font=("Segoe UI", 9),
        )
        self._lbl_state.pack(side="left", padx=(4, 0))

    # ── Body ──────────────────────────────────────────────────────────
    def _build_body(self):
        body = tk.Frame(self, bg=BG)
        body.pack(fill="both", expand=True)
        self._build_left(body)
        tk.Frame(body, bg=BORDER, width=1).pack(side="left", fill="y")
        self._build_right(body)

    # ── Panel izquierdo — tarjetas de categoría ───────────────────────
    def _build_left(self, parent):
        left = tk.Frame(parent, bg=BG, width=390)
        left.pack(side="left", fill="y")
        left.pack_propagate(False)

        # Cabecera del panel
        ph = tk.Frame(left, bg=BG, padx=20, pady=12)
        ph.pack(fill="x")
        tk.Label(
            ph, text="ELEMENTOS A LIMPIAR",
            bg=BG, fg=MUTED, font=("Segoe UI", 8, "bold"),
        ).pack(side="left")
        tk.Frame(left, bg=BORDER, height=1).pack(fill="x")

        # Scroll
        outer  = tk.Frame(left, bg=BG)
        outer.pack(fill="both", expand=True)
        canvas = tk.Canvas(outer, bg=BG, bd=0, highlightthickness=0)
        sb     = tk.Scrollbar(outer, orient="vertical", command=canvas.yview, width=5)
        inner  = tk.Frame(canvas, bg=BG)

        inner.bind(
            "<Configure>",
            lambda e: canvas.configure(scrollregion=canvas.bbox("all")),
        )
        canvas.create_window((0, 0), window=inner, anchor="nw", width=370)
        canvas.configure(yscrollcommand=sb.set)
        canvas.pack(side="left", fill="both", expand=True)
        sb.pack(side="right", fill="y")
        canvas.bind_all(
            "<MouseWheel>",
            lambda e: canvas.yview_scroll(-1 * (e.delta // 120), "units"),
        )

        tk.Frame(inner, bg=BG, height=6).pack()
        for cat in self.categories:
            self._make_card(inner, cat)
        tk.Frame(inner, bg=BG, height=6).pack()

    def _make_card(self, parent, cat: dict):
        var = tk.BooleanVar(value=cat["default"])
        self.vars[cat["id"]] = var

        # ── Contenedor exterior (margen lateral) ─────────────────────
        wrapper = tk.Frame(parent, bg=BG, padx=12, pady=2)
        wrapper.pack(fill="x")

        card = tk.Frame(wrapper, bg=SURFACE2, cursor="hand2")
        card.pack(fill="x")

        # Barra lateral de acento (3 px)
        bar = tk.Frame(card, width=3, bg=ACCENT if var.get() else MUTED)
        bar.pack(side="left", fill="y")

        # Contenido interior
        body = tk.Frame(card, bg=SURFACE2, padx=14, pady=10)
        body.pack(side="left", fill="both", expand=True)

        row = tk.Frame(body, bg=SURFACE2)
        row.pack(fill="x")

        lbl_name = tk.Label(
            row, text=cat["label"],
            bg=SURFACE2, fg=TEXT,
            font=("Segoe UI", 10),
            anchor="w",
        )
        lbl_name.pack(side="left", fill="x", expand=True)

        if not cat["default"]:
            tk.Label(
                row, text="OFF",
                bg=SURFACE2, fg=MUTED, font=("Segoe UI", 7, "bold"),
            ).pack(side="right", padx=(0, 8))

        toggle = ToggleSwitch(row, var, bg=SURFACE2)
        toggle.pack(side="right")

        lbl_desc = tk.Label(
            body, text=cat["desc"],
            bg=SURFACE2, fg=MUTED2,
            font=("Segoe UI", 8),
            anchor="w", justify="left",
        )
        lbl_desc.pack(fill="x", pady=(3, 0))

        # ── Hover ─────────────────────────────────────────────────────
        hover_widgets = [card, body, row, lbl_name, lbl_desc]

        def _enter(_):
            for w in hover_widgets:
                w.configure(bg=SURFACE3)
            toggle.set_bg(SURFACE3)

        def _leave(_):
            for w in hover_widgets:
                w.configure(bg=SURFACE2)
            toggle.set_bg(SURFACE2)

        for w in hover_widgets + [toggle]:
            w.bind("<Enter>", _enter)
            w.bind("<Leave>", _leave)

        # Clic en toda la tarjeta activa/desactiva
        def _click(_):
            var.set(not var.get())

        for w in [card, body, row, lbl_name, lbl_desc]:
            w.bind("<Button-1>", _click)

        # Actualizar barra de acento cuando cambia el toggle
        var.trace_add("write", lambda *_: bar.configure(
            bg=ACCENT if var.get() else MUTED
        ))

    # ── Panel derecho — terminal ──────────────────────────────────────
    def _build_right(self, parent):
        right = tk.Frame(parent, bg=SURFACE)
        right.pack(side="right", fill="both", expand=True)

        # Barra superior del terminal
        bar = tk.Frame(right, bg=SURFACE, padx=16, pady=10)
        bar.pack(fill="x")

        dot_row = tk.Frame(bar, bg=SURFACE)
        dot_row.pack(side="left")
        for col, tip in (("#ef4444",""), ("#f59e0b",""), ("#10b981","")):
            tk.Label(
                dot_row, text="●", bg=SURFACE, fg=col,
                font=("Segoe UI", 9),
            ).pack(side="left", padx=2)

        tk.Label(
            bar, text="  Log de depuración",
            bg=SURFACE, fg=MUTED2, font=("Segoe UI", 9),
        ).pack(side="left")

        tk.Button(
            bar, text="Limpiar",
            bg=SURFACE, fg=MUTED, font=("Segoe UI", 8),
            bd=0, cursor="hand2", relief="flat",
            activebackground=SURFACE, activeforeground=TEXT,
            command=self._clear_log,
        ).pack(side="right")

        tk.Frame(right, bg=BORDER, height=1).pack(fill="x")

        mono = mono_font()
        self.log_widget = tk.Text(
            right, bg=LOG_BG, fg=TEXT,
            font=(mono, 9),
            bd=0, padx=16, pady=14,
            state="disabled", wrap="word",
            insertbackground=TEXT,
            selectbackground="#1e2d47",
        )
        self.log_widget.pack(fill="both", expand=True)

        # Tags de color
        self.log_widget.tag_config(C_OK,   foreground=C_OK)
        self.log_widget.tag_config(C_WARN, foreground=C_WARN)
        self.log_widget.tag_config(C_ERR,  foreground=C_ERR)
        self.log_widget.tag_config(C_INFO, foreground=C_INFO)
        self.log_widget.tag_config(MUTED,  foreground=MUTED)
        self.log_widget.tag_config(MUTED2, foreground=MUTED2)
        self.log_widget.tag_config(ACCENT_L, foreground=ACCENT_L)
        self.log_widget.tag_config(
            "bold",    font=(mono, 9, "bold"), foreground=TEXT,
        )
        self.log_widget.tag_config(
            "summary", font=(mono, 9, "bold"), foreground=ACCENT_L,
        )
        self.log_widget.tag_config("ts", foreground="#1e3050")

    # ── Footer ────────────────────────────────────────────────────────
    def _build_footer(self):
        foot = tk.Frame(self, bg=SURFACE, padx=28, pady=14)
        foot.pack(fill="x")

        self.lbl_status = tk.Label(
            foot, text="Listo.",
            bg=SURFACE, fg=MUTED2, font=("Segoe UI", 9),
        )
        self.lbl_status.pack(side="left")

        # Botón principal
        self.btn_run = tk.Button(
            foot,
            text="   ▶  Iniciar limpieza   ",
            bg=ACCENT, fg="white",
            font=("Segoe UI", 10, "bold"),
            bd=0, padx=18, pady=9,
            cursor="hand2", relief="flat",
            activebackground=ACCENT_D, activeforeground="white",
            command=self._start,
        )
        self.btn_run.pack(side="right")

        # Separador visual
        tk.Frame(foot, bg=BORDER, width=1).pack(side="right", fill="y", padx=16, pady=2)

        tk.Button(
            foot, text="Deseleccionar todo",
            bg=SURFACE, fg=MUTED2, font=("Segoe UI", 9),
            bd=0, cursor="hand2", relief="flat",
            activebackground=SURFACE, activeforeground=TEXT,
            command=lambda: [v.set(False) for v in self.vars.values()],
        ).pack(side="right", padx=(0, 10))

        tk.Button(
            foot, text="Seleccionar todo",
            bg=SURFACE, fg=MUTED2, font=("Segoe UI", 9),
            bd=0, cursor="hand2", relief="flat",
            activebackground=SURFACE, activeforeground=TEXT,
            command=lambda: [v.set(True) for v in self.vars.values()],
        ).pack(side="right", padx=(0, 4))

    # ── Log ───────────────────────────────────────────────────────────
    def _log(self, msg: str, color: str = TEXT):
        def _do():
            ts = datetime.datetime.now().strftime("%H:%M:%S")
            w  = self.log_widget
            w.configure(state="normal")
            w.insert("end", f"[{ts}] ", "ts")
            w.insert("end", f"{msg}\n", color)
            w.see("end")
            w.configure(state="disabled")
        self.after(0, _do)

    def _log_sep(self, char: str = "─", color: str = MUTED):
        self._log(char * 50, color)

    def _clear_log(self):
        self.log_widget.configure(state="normal")
        self.log_widget.delete("1.0", "end")
        self.log_widget.configure(state="disabled")

    # ── Animación de estado ───────────────────────────────────────────
    def _animate_status(self):
        if not self.running:
            self._dot.configure(fg=C_OK)
            return
        colors = [ACCENT, ACCENT_L, "white", ACCENT_L, ACCENT, MUTED]
        self._anim_idx = (self._anim_idx + 1) % len(colors)
        self._dot.configure(fg=colors[self._anim_idx])
        self.after(380, self._animate_status)

    # ── Actualizar contador de espacio liberado ───────────────────────
    def _add_freed(self, freed: int):
        self._freed_total += freed
        self.after(0, lambda: self._lbl_freed.configure(
            text=format_size(self._freed_total)
        ))

    # ── Diálogo papelera ──────────────────────────────────────────────
    def _ask_recycle_bin(self) -> str:
        result: dict[str, str] = {"val": "skip"}
        done = threading.Event()

        def _show():
            dlg = RecycleBinDialog(self)
            self.wait_window(dlg)
            result["val"] = dlg.result
            done.set()

        self.after(0, _show)
        done.wait()
        return result["val"]

    # ── Limpieza ──────────────────────────────────────────────────────
    def _start(self):
        selected = [c for c in self.categories if self.vars[c["id"]].get()]
        if not selected:
            self._log("No hay ningún elemento seleccionado.", C_WARN)
            return
        self._freed_total = 0
        self._lbl_freed.configure(text="0 B", fg=ACCENT_L)
        self._set_running(True)
        threading.Thread(target=self._run, args=(selected,), daemon=True).start()

    def _set_running(self, state: bool):
        self.running = state
        if state:
            self.btn_run.configure(
                state="disabled",
                text="   ◌  Limpiando...   ",
                bg="#2d1f5e", fg=MUTED2,
            )
            self._lbl_state.configure(text="Limpiando", fg=C_WARN)
            self._animate_status()
        else:
            self.btn_run.configure(
                state="normal",
                text="   ▶  Iniciar limpieza   ",
                bg=ACCENT, fg="white",
            )
            self._lbl_state.configure(text="Completado", fg=C_OK)

    def _run(self, selected: list[dict]):
        self._log_sep("═", ACCENT_L)
        self._log(f"  LIMPIEZA INICIADA — {len(selected)} elemento(s)", "summary")
        self._log_sep("═", ACCENT_L)

        stats: list[dict] = []

        for i, cat in enumerate(selected, 1):
            self._log(f"[{i}/{len(selected)}]  {cat['label']}", "bold")
            d, l, freed = self._process(cat)
            self._add_freed(freed)
            stats.append({
                "label":   cat["label"],
                "deleted": max(d, 0),
                "locked":  l,
                "freed":   freed,
            })

        self._show_summary(stats)
        self.after(0, lambda: self._set_running(False))

    # ── Resumen final ─────────────────────────────────────────────────
    def _show_summary(self, stats: list[dict]):
        total_del   = sum(s["deleted"] for s in stats)
        total_lock  = sum(s["locked"]  for s in stats)
        total_freed = sum(s["freed"]   for s in stats)

        self._log_sep("═", ACCENT_L)
        self._log("  RESUMEN DE LIMPIEZA", "summary")
        self._log_sep("─")

        for s in stats:
            if s["deleted"] == 0 and s["freed"] == 0:
                self._log(f"  {s['label']}", MUTED2)
                self._log("    — Sin archivos que borrar o se omitió.", MUTED)
            else:
                self._log(f"  {s['label']}", TEXT)
                self._log(
                    f"    → {s['deleted']} archivos  ·  {format_size(s['freed'])} liberados",
                    C_OK if s["locked"] == 0 else C_WARN,
                )
                if s["locked"]:
                    self._log(f"    ⚠  {s['locked']} bloqueados (en uso)", C_WARN)

        self._log_sep("─")
        self._log(f"  ESPACIO TOTAL LIBERADO  →  {format_size(total_freed)}", "summary")
        self._log(f"  Archivos eliminados     →  {total_del}", C_OK)
        if total_lock:
            self._log(
                f"  Bloqueados              →  {total_lock}  (se liberan al reiniciar)",
                C_WARN,
            )
        else:
            self._log("  Sin archivos bloqueados  →  limpieza total", C_OK)
        self._log_sep("═", ACCENT_L)

        status_txt = f"Liberado: {format_size(total_freed)}  ·  {total_del} archivos"
        self.after(0, lambda: self.lbl_status.configure(text=status_txt))
        self.after(0, lambda: self._lbl_freed.configure(
            text=format_size(total_freed), fg=C_OK,
        ))

    # ── Procesado por tipo ────────────────────────────────────────────
    def _process(self, cat: dict) -> tuple[int, int, int]:
        t     = cat["type"]
        bat   = cat.get("bat", "")
        paths = cat.get("paths", [])

        if t == "bat_folder":
            self._log(f"    → {bat}", MUTED)
            ok, freed, err = run_bat_measured(bat, paths)
            if not ok:
                self._log(f"      ! {err}", C_WARN)
                return 0, 0, freed
            self._log(f"      ✓ {format_size(freed)} liberados", C_OK)
            return 1, 0, freed

        elif t == "bat_command":
            self._log(f"    → {bat}", MUTED)
            ok, err = run_bat(bat)
            self._log("      ✓ Completado" if ok else f"      ! {err}",
                      C_OK if ok else C_WARN)
            return (1, 0, 0) if ok else (0, 0, 0)

        elif t == "recycle_bin":
            self._log("    → Esperando respuesta del usuario...", MUTED)
            action = self._ask_recycle_bin()
            if action == "delete":
                ok, freed = empty_recycle_bin()
                self._log(
                    f"      ✓ Papelera vaciada · {format_size(freed)}" if ok
                    else "      ! Error al vaciar la papelera.",
                    C_OK if ok else C_WARN,
                )
                return (1, 0, freed) if ok else (0, 0, 0)
            self._log("      — Omitido por el usuario.", MUTED)
            return 0, 0, 0

        elif t == "event_logs":
            self._log(f"    → {bat or 'wevtutil'}", MUTED)
            cleared, failed, freed = clear_event_logs()
            self._log(
                f"      ✓ {cleared} registros limpiados · "
                f"{failed} protegidos · {format_size(freed)}",
                C_OK,
            )
            return cleared, failed, freed

        return 0, 0, 0
