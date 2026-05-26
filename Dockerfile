# ============================================================
# ClearTool — Dockerfile multi-stage para Linux
# ============================================================
# ClearTool es una app de escritorio Windows 11 (Tauri + Rust + React).
# Este Dockerfile ofrece dos modos:
#
#   1. WEB MODE (default) — Sirve el frontend React como app web estática.
#      El backend Rust (Win32, registro, debloat) NO funciona en Linux.
#      Ideal para demo, desarrollo UI, o pruebas del frontend.
#
#   2. TAURI MODE — Construye el binario Tauri completo para Linux.
#      Requiere X11/Wayland para mostrar la GUI. Las funciones Windows
#      (limpieza de caché, debloat, registro) devuelven "no disponible".
#
# Uso WEB MODE:
#   docker build -t cleartool:web .
#   docker run -p 8080:80 cleartool:web
#   → abre http://localhost:8080
#
# Uso TAURI MODE:
#   docker build --target tauri-runtime -t cleartool:tauri .
#   docker run --rm -e DISPLAY=$DISPLAY \
#     -v /tmp/.X11-unix:/tmp/.X11-unix \
#     cleartool:tauri
# ============================================================

# ────────────────────────────────────────────────────────────
# Stage 1: frontend-builder — instala Node deps y compila Vite
# ────────────────────────────────────────────────────────────
FROM node:20-alpine AS frontend-builder

WORKDIR /app

# Instalar dependencias del frontend
COPY package.json package-lock.json ./
RUN npm ci --frozen-lockfile

# Copiar el resto del código frontend
COPY tsconfig.json tsconfig.node.json vite.config.ts ./
COPY index.html ./
COPY public/ ./public/
COPY src/ ./src/
COPY postcss.config.js tailwind.config.js ./

# Build del frontend → /app/dist
RUN npm run build

# ────────────────────────────────────────────────────────────
# Stage 2: rust-builder — compila el backend Rust (Tauri)
# ────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS rust-builder

# Dependencias de sistema para compilar Tauri en Linux
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Instalar Tauri CLI
RUN cargo install tauri-cli --version "^2" --locked

WORKDIR /app

# Copiar Cargo files primero para aprovechar cache de Docker
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./src-tauri/
COPY src-tauri/build.rs ./src-tauri/
COPY src-tauri/tauri.conf.json ./src-tauri/
COPY src-tauri/icons/ ./src-tauri/icons/
COPY src-tauri/capabilities/ ./src-tauri/capabilities/

# Copiar el código Rust
COPY src-tauri/src/ ./src-tauri/src/

# Copiar el frontend ya compilado (Tauri lo embebe en el binario)
COPY --from=frontend-builder /app/dist ./dist/

# Build del binario Tauri para Linux
# --no-bundle = solo compila el binario, sin crear .deb/.AppImage
WORKDIR /app/src-tauri
RUN cargo build --release --lib

# ────────────────────────────────────────────────────────────
# Stage 3: web-runtime — servidor nginx para el frontend
# ────────────────────────────────────────────────────────────
FROM nginx:alpine AS web-runtime

# Copiar el frontend compilado al directorio de nginx
COPY --from=frontend-builder /app/dist /usr/share/nginx/html

# Config nginx para SPA (redirigir todo a index.html)
RUN echo 'server { \
    listen 80; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

# ────────────────────────────────────────────────────────────
# Stage 4: tauri-runtime — imagen con el binario Tauri + X11
# ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS tauri-runtime

# Dependencias de runtime para Tauri (GTK, WebKit2GTK)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    libappindicator3-1 \
    librsvg2-2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiar el binario compilado
COPY --from=rust-builder /app/src-tauri/target/release/libcleartool_lib.so ./lib/
COPY --from=rust-builder /app/src-tauri/target/release/cleartool ./bin/

# El binario Tauri necesita acceder a los assets del frontend
COPY --from=frontend-builder /app/dist ./dist/

# Variables de entorno para X11
ENV DISPLAY=:0
ENV XDG_RUNTIME_DIR=/tmp/runtime
ENV WAYLAND_DISPLAY=

# Crear directorio de runtime
RUN mkdir -p /tmp/runtime

# El entrypoint ejecuta el binario Tauri
# Nota: requiere montar el socket X11 del host:
#   docker run --rm -e DISPLAY=$DISPLAY \
#     -v /tmp/.X11-unix:/tmp/.X11-unix \
#     cleartool:tauri
ENTRYPOINT ["/app/bin/cleartool"]
