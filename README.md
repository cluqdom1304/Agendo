# Agendo

**Tu agenda y tus apuntes en un solo sitio, rápida, bonita y 100 % tuya.**

Agenda y editor de notas de escritorio para Windows 11. Combina la estructura de un editor plano (secciones «POR HACER» y «AGENDA» con fechas descendentes) con checkboxes, etiquetas por asignatura, vistas de calendario, recordatorios nativos y una estética inspirada en Apple HIG.

**Principios:** 100 % local, sin cuentas, sin telemetría, sin red, ligera y con recordatorios que no se pierden.

---

## Características

- **Tareas y notas:** crea, edita, completa y elimina (papelera de 30 días y deshacer de 5 s).
- **POR HACER y AGENDA:** tareas sin fecha con orden manual y bloques por día con hora opcional.
- **Etiquetas:** una por asignatura, con color, contador de pendientes y filtro en la barra lateral.
- **Editor de notas:** negrita, cursiva, código, enlaces y listas, con guardado automático.
- **Búsqueda instantánea** sobre títulos y notas (SQLite FTS5).
- **Calendario:** vistas Lista, Día, Semana y Mes; crea eventos arrastrando un tramo horario.
- **Recordatorios nativos de Windows 11:** varios por tarea; los perdidos con el equipo apagado se recuperan al arrancar.
- **Atajos de teclado** completos (`Ctrl+K` buscar, `Ctrl+N` nueva tarea, `Ctrl+1/2/3/4` vistas…).
- **Copia de seguridad y restauración** manual con validación de integridad.
- **Mica / Acrylic**, tema claro y oscuro.
- **Privacidad:** tus datos viven en un archivo SQLite local. Nada sale de tu equipo.

---

## Stack tecnológico

| Capa | Tecnología |
|------|------------|
| Aplicación de escritorio | Tauri v2 (Rust) |
| Frontend | Svelte 5 (runes) + Vite + TypeScript |
| Estilos | Tailwind CSS v4 + tokens CSS propios (Apple HIG) |
| Editor de notas | TipTap Core (ProseMirror) |
| Base de datos | SQLite vía `rusqlite` (bundled), modo WAL, FTS5 para búsqueda |
| Recordatorios | Tokio (scheduler) + `tauri-plugin-notification` |
| Efectos de ventana | Mica / Acrylic nativos de Windows 11 |
| Tests | `cargo test`, Vitest + Testing Library, WebdriverIO + tauri-driver (E2E) |
| Calidad | rustfmt, clippy, ESLint, Prettier, svelte-check |

Detalle completo en `docs/SPEC.md` (§3) y decisiones en `DECISIONS.md`.

---

## Requisitos de desarrollo (Windows 11)

- **Rust** estable mediante `rustup`.
- **Node.js** LTS vigente.
- **Visual Studio Build Tools** con la carga de trabajo "Desarrollo para escritorio con C++" (MSVC).
- **WebView2 Runtime** (incluido en Windows 11; instalar manualmente si falta).

---

## Puesta en marcha

```bash
git clone <url-del-repositorio> agendo
cd agendo
npm install
npm run tauri dev
```

### Comandos habituales

| Comando | Descripción |
|---------|-------------|
| `npm run tauri dev` | Ejecuta la app en modo desarrollo |
| `npm run tauri build` | Genera el ejecutable y el instalador de release |
| `npm run test` | Tests de frontend (Vitest) |
| `cargo test` (en `src-tauri/`) | Tests de backend |
| `npm run lint` / `npm run check` | Linters y comprobación de tipos |
| `npm run size` | Verifica el presupuesto de tamaño del bundle |

---

## Documentación del proyecto

| Archivo | Para qué sirve |
|---------|----------------|
| `docs/SPEC.md` | Especificación completa: requisitos, arquitectura, datos, diseño, hoja de ruta |
| `PROGRESS.md` | Estado actual y lista de tareas por fase |
| `DECISIONS.md` | Registro de decisiones de diseño y versiones fijadas |
| `BLOCKERS.md` | Bloqueos y decisiones pendientes de producto |
| `CLAUDE.md` | Reglas de trabajo para el agente de desarrollo |
| `docs/manual-pruebas.md` | Checklist de validación manual en Windows 11 |

---

## Contribuir

Las contribuciones son bienvenidas. Antes de abrir un PR ejecuta la batería de verificación: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run lint`, `npm run check`, `npm run test`, `npm run build` y `npm run size`. Los commits siguen [Conventional Commits](https://www.conventionalcommits.org/es/).

## Licencia

Distribuido bajo la licencia [MIT](LICENSE.md). © 2026 Cristian Luque Domínguez.
