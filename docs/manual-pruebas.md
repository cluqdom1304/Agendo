# Manual de pruebas en Windows 11

Checklist de validación humana. El agente lo rellena con los ítems `[H]` de cada fase, pero **nunca** marca un ítem como superado. Solo lo hace una persona.

**Equipo de referencia:** registrar modelo, RAM, versión de Windows 11 (build), versión de WebView2 Runtime y si el equipo usa batería o enchufe.

**Cómo marcar:** `[x]` = superado · `[!]` = fallo (abrir entrada en BLOCKERS.md con el detalle) · `[ ]` = no probado.

**Preparación común (antes de cada fase):**
1. Compilar en modo release: `npm run tauri build`.
2. Instalar desde el instalador o ejecutar el binario de `src-tauri/target/release/`.
3. Anotar fecha, build de Windows y commit probado.

---

## Fase 0 — Esqueleto

- [ ] La ventana aparece sin barra nativa, con esquinas redondeadas de 10 px.
- [ ] El efecto Mica se ve en Windows 11. Si no, indicar qué efecto se aplica (Acrylic o sólido).
- [ ] La ventana se arrastra desde la barra de título y no desde los botones.
- [ ] Minimizar, maximizar y cerrar funcionan.
- [ ] Redimensionar respeta el tamaño mínimo de 720×480.
- [ ] Medir N-01: tiempo desde el lanzamiento hasta la primera pantalla pintada (promedio de 5 arranques). Anotar valor.
- [ ] Medir N-02: memoria del proceso principal y suma de procesos WebView2 en reposo tras 60 s. Anotar ambos valores.
- [ ] Anotar tamaño del ejecutable de release (N-03).

---

## Fase 2 — Vista principal

- [ ] Las secciones "POR HACER", "AGENDA" y "Completadas" se distinguen visualmente.
- [ ] Las fechas aparecen en `DD/MM/YYYY` y en orden descendente.
- [ ] Al completar una tarea, la animación es fluida (sin saltos visibles) y la tarea acaba en "Completadas" del mismo día.
- [ ] El checkbox tiene la animación elástica y no hay artefactos.
- [ ] Ctrl+N crea una tarea y el foco queda en el título.
- [ ] El deshacer de 5 s restaura la tarea eliminada.
- [ ] Con `prefers-reduced-motion` activado en Windows, las animaciones desaparecen.
- [ ] La sensación de documento tipo Notepad++ es clara y coherente.

---

## Fase 3 — Editor y búsqueda

- [ ] Negrita, cursiva, código, enlaces y listas se aplican con los atajos indicados.
- [ ] Un enlace `javascript:` o `file:` no se abre ni se guarda.
- [ ] Escribir en una nota no provoca tirones perceptibles.
- [ ] Ctrl+K abre la búsqueda y los resultados se navegan con las flechas y Enter.
- [ ] La búsqueda encuentra palabras con y sin acento.
- [ ] Los sub-puntos aparecen indentados justo debajo de su tarea.

---

## Fase 4 — Recordatorios y notificaciones

- [ ] Un recordatorio "En el momento" muestra una notificación nativa de Windows 11 a la hora indicada (± 5 s).
- [ ] Un recordatorio "15 min antes" dispara a la hora correcta.
- [ ] El recordatorio aparece en el Centro de actividades de Windows.
- [ ] Con la app cerrada, un recordatorio no se pierde: al volver a abrir la app dentro de las 24 h siguientes, se notifica.
- [ ] Tras suspender el equipo y reanudarlo, los recordatorios vencidos se gestionan según F-14.
- [ ] Cambiar la hora de una tarea reprograma su recordatorio.
- [ ] Un recordatorio personalizado se respeta tras reiniciar el equipo.
- [ ] Si se cambia la zona horaria de Windows, los recordatorios disparan a la hora local esperada (anotar comportamiento).
- [ ] Si hay varias notificaciones seguidas, Windows las agrupa o las muestra sin bloquear la app.

---

## Fase 5 — Calendario

- [ ] El selector Lista / Día / Semana / Mes cambia la vista sin recargar la app.
- [ ] La semana empieza el día que Windows tiene configurado.
- [ ] Arrastrar en Día o Semana crea un evento con la hora correcta.
- [ ] Los bloques de cada asignatura usan su color.
- [ ] Los cambios de hora de verano/invierno no desplazan tareas de día.

---

## Fase 6 — Pulido y empaquetado

- [ ] Modo claro y oscuro: contraste legible en todos los textos.
- [ ] La navegación completa es posible solo con teclado.
- [ ] Un lector de pantalla (Narrador de Windows) anuncia los controles principales.
- [ ] Copia de seguridad: se crea el archivo y se puede abrir con otro gestor SQLite.
- [ ] Restauración desde copia: los datos vuelven al estado de la copia.
- [ ] Restauración desde un archivo corrupto se rechaza sin tocar los datos actuales.
- [ ] El instalador instala, desinstala y deja la app funcional tras reinstalar.
- [ ] Sin firma digital, Windows SmartScreen muestra el aviso esperado (documentar).
- [ ] Tabla final de métricas N-01 a N-08 completada en `docs/`.

---

## Registro de sesiones de prueba

| Fecha | Build / commit | Equipo | Fase | Resultado | Responsable |
|-------|----------------|--------|------|-----------|-------------|
| | | | | | |
