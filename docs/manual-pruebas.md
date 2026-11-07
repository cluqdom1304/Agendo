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

- [x] La ventana aparece sin barra nativa, con esquinas redondeadas de 10 px.
- [x] El efecto Mica se ve en Windows 11. Si no, indicar qué efecto se aplica (Acrylic o sólido).
- [x] El efecto que informa `window_set_effect` (`applied`) coincide con lo que se ve al pedir `mica`, `acrylic` y `none` (una vez F0-08 lo conecte a la UI).
- [x] La ventana se arrastra desde la barra de título y no desde los botones.
- [x] Minimizar, maximizar y cerrar funcionan.
- [x] Con el tema de Windows en claro y en oscuro, texto, botones de ventana y fondo son legibles y coherentes (los tokens cambian con `prefers-color-scheme`).
- [x] Con el efecto nativo activo el fondo se ve con tinte translúcido, sin doble desenfoque; sin efecto el fondo es casi opaco.
- [x] Las barras de desplazamiento miden 4 px y solo se ven al pasar el ratón por la zona.
- [x] Redimensionar respeta el tamaño mínimo de 720×480.
- [x] Con `npm run tauri dev`, la terminal muestra las marcas `[startup +X.XXms]` en este orden: `process_start`, `setup_done`, `page_load_started`, `page_load_finished`, `first_paint`. En un build de release no se imprime ninguna.
- [x] Medir N-01: tiempo desde el lanzamiento hasta la primera pantalla pintada (promedio de 5 arranques). N-01 = 1600.836 ms (medidos con las marcas de tiempo `startup`).
- [x] Medir N-02: memoria del proceso principal y suma de procesos WebView2 en reposo tras 60 s. N-02: proceso principal = 6.2 MB; webview = 84.6 MB.
- [x] Anotar tamaño del ejecutable de release (N-03). N-03 = 1.917 KB.

---

## Fase 2 — Vista principal

- [x] Las secciones "POR HACER", "AGENDA" y "Completadas" se distinguen visualmente.
- [x] Las fechas aparecen en `DD/MM/YYYY` y en orden descendente.
- [x] Al completar una tarea, la animación es fluida (sin saltos visibles) y la tarea acaba en "Completadas" del mismo día.
- [x] El checkbox tiene la animación elástica y no hay artefactos.
- [x] Ctrl+N crea una tarea y el foco queda en el título.
- [x] El deshacer de 5 s restaura la tarea eliminada.
- [x] Con `prefers-reduced-motion` activado en Windows, las animaciones desaparecen.
- [x] La sensación de documento tipo Notepad++ es clara y coherente.

---

## Fase 3 — Editor y búsqueda

- [x] Negrita, cursiva, código, enlaces y listas se aplican con los atajos indicados.
- [ ] Un enlace `javascript:` o `file:` no se abre ni se guarda.
- [x] Escribir en una nota no provoca tirones perceptibles.
- [x] Ctrl+K abre la búsqueda y los resultados se navegan con las flechas y Enter.
- [x] La búsqueda encuentra palabras con y sin acento.
- [x] Los sub-puntos aparecen indentados justo debajo de su tarea.

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
- [ ] La campana de cada fila abre el panel de recordatorios; las opciones cuyo momento ya pasó aparecen desactivadas, y el campo «Hora de la tarea» permite poner o quitar la hora.
- [ ] Con la app abierta y en primer plano, al disparar un recordatorio aparece además el aviso «Recordatorio: …» dentro de la app.
- [ ] Una tarea completada o eliminada no avisa; si se reabre o se restaura a tiempo, vuelve a avisar.

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
