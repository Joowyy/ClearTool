/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // Tokens semánticos (resueltos via HSL en globals.css)
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        // Quirófano Cyan — sistema de superficies (lightness whisper-quiet)
        // Todas con <alpha-value> para soportar modificadores `/25` etc.
        surface: {
          canvas: "hsl(var(--surface-canvas) / <alpha-value>)",
          1: "hsl(var(--surface-1) / <alpha-value>)",
          2: "hsl(var(--surface-2) / <alpha-value>)",
          3: "hsl(var(--surface-3) / <alpha-value>)",
          inset: "hsl(var(--surface-inset) / <alpha-value>)",
        },
        ink: {
          primary: "hsl(var(--ink-primary) / <alpha-value>)",
          secondary: "hsl(var(--ink-secondary) / <alpha-value>)",
          tertiary: "hsl(var(--ink-tertiary) / <alpha-value>)",
          muted: "hsl(var(--ink-muted) / <alpha-value>)",
        },
        edge: {
          subtle: "hsl(var(--edge-subtle) / <alpha-value>)",
          DEFAULT: "hsl(var(--edge-default) / <alpha-value>)",
          strong: "hsl(var(--edge-strong) / <alpha-value>)",
          focus: "hsl(var(--edge-focus) / <alpha-value>)",
        },
        signal: {
          cyan: "hsl(var(--signal-cyan) / <alpha-value>)",
          "cyan-press": "hsl(var(--signal-cyan-press) / <alpha-value>)",
          amber: "hsl(var(--signal-amber) / <alpha-value>)",
          red: "hsl(var(--signal-red) / <alpha-value>)",
          "red-press": "hsl(var(--signal-red-press) / <alpha-value>)",
          emerald: "hsl(var(--signal-emerald) / <alpha-value>)",
        },
      },
      fontFamily: {
        sans: [
          '"Geist Variable"',
          "ui-sans-serif",
          "system-ui",
          '"Segoe UI Variable"',
          "Segoe UI",
          "sans-serif",
        ],
        mono: [
          '"JetBrains Mono Variable"',
          "ui-monospace",
          '"Cascadia Mono"',
          "Consolas",
          "monospace",
        ],
      },
      fontSize: {
        // Escala denso/técnica — solo lo necesario
        "2xs": ["10px", { lineHeight: "14px", letterSpacing: "0.01em" }],
        xs: ["11px", { lineHeight: "15px" }],
        sm: ["13px", { lineHeight: "18px" }],
        base: ["14px", { lineHeight: "20px" }],
        md: ["15px", { lineHeight: "22px" }],
        lg: ["17px", { lineHeight: "24px", letterSpacing: "-0.005em" }],
        xl: ["20px", { lineHeight: "28px", letterSpacing: "-0.01em" }],
        "2xl": ["26px", { lineHeight: "32px", letterSpacing: "-0.015em" }],
        "3xl": ["32px", { lineHeight: "38px", letterSpacing: "-0.02em" }],
      },
      borderRadius: {
        none: "0",
        sm: "4px",
        DEFAULT: "6px",
        md: "8px",
        lg: "10px",
        xl: "14px",
        full: "9999px",
      },
      spacing: {
        // Base 4 — escala estricta
        0.5: "2px",
        1: "4px",
        2: "8px",
        3: "12px",
        4: "16px",
        5: "20px",
        6: "24px",
        7: "28px",
        8: "32px",
        10: "40px",
        12: "48px",
        16: "64px",
      },
      transitionTimingFunction: {
        // Decelerated — apropiado para superficies que se asientan
        decel: "cubic-bezier(0.16, 1, 0.3, 1)",
        soft: "cubic-bezier(0.4, 0, 0.2, 1)",
      },
      transitionDuration: {
        120: "120ms",
        160: "160ms",
        200: "200ms",
      },
      animation: {
        "fade-in": "fadeIn 200ms cubic-bezier(0.16, 1, 0.3, 1) both",
        "slide-up": "slideUp 240ms cubic-bezier(0.16, 1, 0.3, 1) both",
        "scan-line": "scanLine 8s linear infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        scanLine: {
          "0%": { transform: "translateY(-100%)" },
          "100%": { transform: "translateY(100vh)" },
        },
      },
    },
  },
  plugins: [],
};
