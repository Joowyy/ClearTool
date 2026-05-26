# docs/v2 — Plan de implementación mascado

Esta carpeta es la **traducción accionable** de los specs en `.claude/specs/v2/`. Mientras que los specs explican "qué y por qué", aquí están los pasos concretos: comandos, snippets, paths exactos, checklists.

## Cómo está organizado

Cada subcarpeta es un **área de trabajo** (un módulo nuevo, un refactor, un grupo de tareas afines). Dentro de cada área:

- `README.md` — objetivo, prerrequisitos, lista de pasos en orden, criterio de done agregado.
- `01-*.md`, `02-*.md`, ... — un archivo por paso. Cada paso es una sesión de trabajo (1-3 horas típicamente).

```
docs/v2/
├── README.md                     ← este archivo
├── 00-orden-de-batalla.md        ← qué hacer primero, qué en paralelo
├── 01-error-handling/            ← M1 (P0)
├── 02-cache-engine/              ← M2 (P0)
├── 03-process-manager/           ← M2 (P0)
├── 04-debloat-catalog/           ← M3 (P1)
├── 05-startup-manager/           ← M3 (P1)
├── 06-boot-cleanup/              ← M3 (P1)
├── 07-disk-analyzer/             ← M4 (P2)
├── 08-network-utilities/         ← M4 (P2)
├── 09-privacy-hardening/         ← M3 (P1)
├── 10-app-reset/                 ← M3 (P2)
├── 11-ui-refactor/               ← M4 (P2)
├── 12-distribution/              ← M5 (P1)
└── 13-testing-ci/                ← Continuo
```

## Estructura de un paso (plantilla)

Todos los archivos de paso siguen este formato:

```markdown
# Paso N — Título corto

**Área**: <carpeta>
**Tiempo estimado**: <S/M/L o horas>
**Dependencias**: <pasos previos>

## Qué hacemos

Una frase clara del objetivo.

## Por qué

El "porqué" técnico/UX en 2-3 líneas.

## Archivos que tocamos

Lista de paths con (nuevo) o (modificado).

## Cómo

Pasos numerados. Code skeleton listo para pegar.

## Criterio de done

- [ ] Checklist verificable.
```

## Por dónde empezar

1. Lee `00-orden-de-batalla.md` para entender el orden global.
2. Abre el README de la primera carpeta (`01-error-handling/`).
3. Sigue los pasos en orden dentro de esa carpeta.
4. Cuando termines un paso, marca el checkbox al final.
5. Cuando termines un área, marca el agregado en su README.

## Vinculación con los specs

Cada paso enlaza al spec correspondiente en `.claude/specs/v2/` cuando aplica. Si el spec dice algo distinto del paso, **el paso gana** (es más concreto). Si descubres una diferencia importante, actualiza el spec.

## Vinculación con `docs/fixes/`

Los `docs/fixes/` son **post-mortem** de bugs ya arreglados (sesiones pasadas). Los `docs/v2/` son **forward-looking** para trabajo futuro. No mezclar contenido.
