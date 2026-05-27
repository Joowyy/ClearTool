# 05 — THREE.WebGLRenderer Context Lost durante análisis

> **Severidad:** 🟡 P1 — el canvas 3D del Home se "pierde" cuando el
> análisis de caché está corriendo. Ruido en consola + posible
> congelado visual al volver al Home.

## 1. Problema

DevTools repite:

```
THREE.WebGLRenderer: Context Lost.
THREE.WebGLRenderer: Context Lost.
THREE.WebGLRenderer: Context Lost.
THREE.WebGLRenderer: Context Lost.
THREE.WebGLRenderer: Context Lost.
```

Pasa cuando:
- El módulo Caché está analizando o limpiando (CPU/IO altos).
- El usuario tabbea fuera de la app y vuelve.
- Otras pestañas del navegador (en dev mode con Vite) compiten por la
  GPU.

## 2. Causa raíz

ClearTool usa Three.js / R3F para el dashboard del Home. El navegador
(`webview2`) recicla contextos WebGL cuando detecta presión de memoria
GPU. Si la app no escucha `webglcontextlost` ni pausa el render loop
cuando no está visible, cada pérdida queda como warning y la escena
se cae.

Además, el Home renderiza permanentemente aunque el usuario esté en
otra pestaña (Caché, Debloat, etc.). Eso desperdicia GPU y agrava la
presión.

## 3. Fix propuesto

### 3.1 Pausar render cuando el componente no está visible

Three.js / R3F deja exponer `frameloop="demand"` que sólo renderiza
cuando algo cambia. Si Home usa R3F, en su `<Canvas>`:

```tsx
<Canvas
  frameloop="demand"
  gl={{
    powerPreference: "low-power",
    antialias: false,
    alpha: true,
    preserveDrawingBuffer: false,
  }}
  onCreated={({ gl }) => {
    gl.domElement.addEventListener("webglcontextlost", (e) => {
      e.preventDefault();
      console.warn("WebGL context lost — pausando render.");
    });
    gl.domElement.addEventListener("webglcontextrestored", () => {
      console.info("WebGL context restored.");
    });
  }}
>
  {/* escena */}
</Canvas>
```

Y disparar `invalidate()` (de R3F) cuando llegue data nueva que cambie
la escena (un summary nuevo, una métrica cambia, etc.).

### 3.2 Desmontar el Canvas si Home no está visible

React Router con lazy loading ya desmonta la página al navegar fuera,
así que **Home no debería estar renderizando si el usuario está en
Caché**. Verificar que el `Canvas` no esté en `AppShell` ni en un
provider global.

Si está en `AppShell`, moverlo al cuerpo de `HomePage` y dejar que
`React.lazy` se encargue de descartarlo.

### 3.3 Fallback 2D cuando WebGL no esté disponible

Detectar fallo y mostrar gráfico simple:

```tsx
const [webglOk, setWebglOk] = useState(true);

useEffect(() => {
  try {
    const c = document.createElement("canvas");
    const gl = c.getContext("webgl2") || c.getContext("webgl");
    setWebglOk(!!gl);
  } catch {
    setWebglOk(false);
  }
}, []);

return webglOk ? <ThreeDashboard /> : <FallbackBarsDashboard />;
```

### 3.4 Reducir uso de GPU en background

En settings, añadir un toggle "Modo bajo consumo" que:

- Pone `frameloop="never"` en el Canvas (sólo se renderiza al `invalidate()`).
- Desactiva post-processing.
- Sube el `dpr` máximo a 1 (no usa retina).

Por defecto: ON (la app es una utility, no necesita 60 fps en el dashboard).

### 3.5 Opción nuclear — quitar el 3D del dashboard

Si el usuario lo prefiere y el dashboard 3D no aporta valor real
(decoración), considerar reemplazarlo por una vista 2D con cards y
sparklines. Eso elimina el problema entero, ahorra 200 KB de JS bundle
y acelera el primer paint.

**Recomendación**: empezar por 3.1 + 3.2 + 3.4. Si después de un par de
sesiones de prueba el warning persiste, ir a 3.5.

## 4. Criterio de done

- [ ] `THREE.WebGLRenderer: Context Lost` aparece como mucho UNA vez
      por sesión (la inicial cuando el navegador pierde el contexto al
      cambiar de tab), no repetidamente.
- [ ] Navegar a Caché → Limpiar → volver a Home no provoca el warning.
- [ ] El dashboard del Home no consume CPU/GPU mientras el usuario está
      en otra pestaña.
- [ ] Si WebGL falla, el Home no queda en blanco — muestra fallback.

## 5. Riesgos / efectos secundarios

- `frameloop="demand"` puede hacer que animaciones que dependen de
  `useFrame` no corran sin `invalidate()`. Hay que auditar la escena
  del dashboard para confirmar qué animaciones quedan.
- `powerPreference: "low-power"` selecciona la GPU integrada en
  portátiles con dGPU. En desktops no cambia nada. Es lo correcto
  para una utility — no necesitamos la NVIDIA al máximo.

## 6. Recursos

- `webglcontextlost` event: https://registry.khronos.org/webgl/specs/latest/1.0/#5.15.2
- R3F `frameloop` modes: https://docs.pmnd.rs/react-three-fiber/api/canvas#render-modes
- Three.js GPU power preference: https://threejs.org/docs/#api/en/renderers/WebGLRenderer
