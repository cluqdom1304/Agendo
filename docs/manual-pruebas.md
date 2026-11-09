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

- [x] Un recordatorio "En el momento" muestra una notificación nativa de Windows 11 a la hora indicada (± 5 s).
- [x] Un recordatorio "15 min antes" dispara a la hora correcta.
- [x] El recordatorio aparece en el Centro de actividades de Windows.
- [x] Con la app cerrada, un recordatorio no se pierde: al volver a abrir la app dentro de las 24 h siguientes, se notifica.
- [ ] Tras suspender el equipo y reanudarlo, los recordatorios vencidos se gestionan según F-14.
- [ ] Cambiar la hora de una tarea reprograma su recordatorio.
- [ ] Un recordatorio personalizado se respeta tras reiniciar el equipo.
- [ ] Si se cambia la zona horaria de Windows, los recordatorios disparan a la hora local esperada (anotar comportamiento).
- [ ] Si hay varias notificaciones seguidas, Windows las agrupa o las muestra sin bloquear la app.
- [x] La campana de cada fila abre el panel de recordatorios; las opciones cuyo momento ya pasó aparecen desactivadas, y el campo «Hora de la tarea» permite poner o quitar la hora.
- [x] Con la app abierta y en primer plano, al disparar un recordatorio aparece además el aviso «Recordatorio: …» dentro de la app.
- [ ] Una tarea completada o eliminada no avisa; si se reabre o se restaura a tiempo, vuelve a avisar.

---

## Fase 5 — Calendario

- [ ] El selector Lista / Día / Semana / Mes cambia la vista sin recargar la app.
- [ ] La semana empieza el día que Windows tiene configurado.
- [ ] Arrastrar en Día o Semana crea un evento con la hora correcta.
- [ ] Los bloques de cada asignatura usan su color. *(No disponible aún: requiere el almacén de etiquetas; ver PROGRESS § Fase 5, desviaciones.)*
- [ ] Los cambios de hora de verano/invierno no desplazan tareas de día.
- [ ] La vista elegida se recuerda al cerrar y volver a abrir la app, y `Ctrl+1/2/3/4` cambian de vista.
- [ ] Al pulsar una tarea del calendario se vuelve a la lista con esa tarea seleccionada; el número de un día del mes abre la vista Día.
- [ ] La línea roja de la hora actual aparece en Día y Semana (bolita en la columna de hoy, la línea sigue hasta el final de la semana), avanza sola con el reloj y no aparece en Mes.
- [ ] Abrir una vista de calendario por primera vez no produce parpadeos ni retraso apreciable (carga diferida).

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
- [ ] Ajustes → Tema: «Sistema», «Claro» y «Oscuro» cambian la apariencia al instante y se conservan al reiniciar.
- [ ] Ajustes → Efecto de la ventana: Mica, Acrílico y Ninguno se aplican; si el equipo no admite uno, la app avisa de cuál usa.
- [ ] Ajustes → Iniciar con Windows está desactivado en una instalación nueva; al activarlo la app arranca al iniciar sesión y al desactivarlo deja de hacerlo (`Administrador de tareas → Inicio`).
- [ ] Copia de seguridad: el cuadro «Guardar como» propone `MiAgenda-AAAA-MM-DD.sqlite` y sobrescribir pide confirmación de Windows.
- [ ] Restaurar: el aviso explica que se sustituyen los datos y que los actuales quedan en `agenda.before-restore.sqlite`; tras restaurar, lista, calendario y recordatorios muestran los datos de la copia sin reiniciar.
- [ ] Restaurar un `.sqlite` ajeno (otra aplicación) o de una versión más nueva se rechaza con un mensaje claro.
- [ ] Calendario con teclado: `Tab` llega a cada columna de día, `Intro` abre «Nueva tarea» y la tarea se guarda a la hora indicada.
- [ ] Contraste real sobre Mica en tema claro y oscuro (los tokens cumplen AA sobre un fondo supuesto; aquí se comprueba con el fondo real).
- [ ] E2E (`npm run e2e`, requiere `tauri-driver` y `msedgedriver`, ver `e2e/README.md`): los tres ficheros de `e2e/specs/` pasan.
- [ ] Medición N-01 (arranque), N-02 (RAM) y N-07 (precisión de recordatorios) registrada en `docs/metricas.md`.
- [ ] Tabla final de métricas N-01 a N-08 completada en `docs/` (el agente rellena N-03 a N-06; las demás son `[H]`).

---

## Mejoras posteriores a la Fase 6

- [ ] El botón «Nueva tarea» abre el diálogo; Ctrl+N sigue creando una fila en línea.
- [ ] En el diálogo se crea una tarea con fecha, hora, descripción y recordatorios (preestablecidos y a hora concreta) y todo queda guardado.
- [ ] Un aviso periódico (p. ej. cada hora, 3 veces) suena, se queda en gris como «Avisado» y aparece el siguiente hasta completar la serie.
- [ ] Una serie «hasta una fecha» no suena después de esa fecha.
- [ ] Con la app cerrada durante varias repeticiones, al abrirla no llegan avisos atrasados en tromba.
- [ ] Una serie sigue tras reiniciar la aplicación.

---

## Detalle de tarea y duración (D-33)

- [ ] Pulsar una tarea en el calendario (día, semana o mes) abre su vista de detalle, no la lista.
- [ ] En el detalle: «Marcar como hecha» / «Reabrir», «Editar» (abre el diálogo de nueva tarea relleno) y «Eliminar»; `Supr` elimina salvo al escribir en las notas; «Deshacer» aparece tras eliminar desde el calendario.
- [ ] Las notas con formato se editan en el detalle y se guardan.
- [ ] En el diálogo, con fecha y hora aparecen «Sin duración», 15 min, 30 min, 1 h y «Termina a las»; la duración queda guardada y el bloque del calendario tiene esa altura.
- [ ] Arrastrar un tramo en el calendario crea la tarea con esa duración; un clic simple no fija duración.
- [ ] Una tarea hecha no ofrece recordatorios (lista, detalle ni diálogo) y no suena ningún aviso; al reabrirla vuelven los pendientes.
- [ ] En la lista, las tareas hechas se ven más apagadas y «Completadas» queda en gris con su contador.

## Etiquetas (D-34)
- [ ] La barra lateral se despliega y pliega con su botón de la barra superior (animación fluida) y empieza plegada si la ventana mide menos de 900 px; con un filtro activo el botón muestra un punto.
- [ ] Crear una etiqueta con un color predefinido, con el selector nativo y escribiendo `#RRGGBB`; un código inválido no deja guardar.
- [ ] Las etiquetas salen siempre en el mismo orden, con su contador (0 si no tienen tareas) y «Sin etiqueta» al final.
- [ ] Desmarcar una casilla oculta esas tareas en la lista y en el calendario; «Todas», «Ninguna» y «Solo» funcionan; ocultarlo todo muestra el aviso con «Mostrar todas».
- [ ] En el diálogo de tarea se elige o se quita la etiqueta; el chip aparece en la fila de la lista y en el detalle.
- [ ] En el calendario el evento toma el color de la etiqueta y muestra su nombre pequeño en semana, día, mes y «Todo el día»; el texto se lee con colores muy claros y muy oscuros, en tema claro y oscuro y con Mica.
- [ ] Editar el nombre o el color se refleja al instante; eliminar una etiqueta pide confirmación y deja sus tareas sin etiqueta.
- [ ] Crear una tarea desde el calendario con «Sin etiqueta» oculto muestra el aviso de que el filtro la oculta.
- [ ] En la lista, al pasar el ratón por una tarea sin etiqueta aparece un icono de etiqueta; un clic abre un menú con todas las etiquetas y «Sin etiqueta» (la actual con ✓); elegir una la asigna al instante, y Esc o clic fuera lo cierra. En una tarea etiquetada, el propio chip abre el menú.
- [ ] Arrastrar desde el asa de seis puntos (izquierda de la fila) una tarea sobre una etiqueta de la barra lateral la etiqueta; sobre «Sin etiqueta» se la quita. La barra lateral se abre sola si estaba plegada, la fila de destino se resalta y Esc cancela el arrastre.
- [ ] Lo mismo con los eventos del calendario (semana, día, mes y «Todo el día»): arrastrar un evento a una etiqueta lo etiqueta, y un clic simple sigue abriendo el detalle. Al soltar tras arrastrar no se abre el detalle.
- [ ] El arrastre es fluido (el aviso con el título sigue al puntero sin retraso) y no selecciona texto ni activa arrastres de archivos de Windows.
- [ ] Con la ventana al mínimo (720×480) la barra lateral y el contenido no se solapan ni aparece scroll horizontal.

---

## Versión 1.0.1

- [ ] Editar una etiqueta en la barra lateral: el botón «Guardar» queda dentro del cuadro de edición, también con la barra lateral estrecha.
- [ ] Ajustes → hora por defecto de avisos: se puede elegir y se conserva al reiniciar.
- [ ] Tarea con fecha y sin hora, aviso «1 día antes»: la notificación llega a la hora por defecto del día anterior. Con «1 semana antes», una semana antes. Las opciones en minutos u horas no aparecen.
- [ ] La misma tarea sigue en «Todo el día» del calendario, no a la hora por defecto.
- [ ] Cambiar la hora por defecto mueve los avisos pendientes de tareas sin hora, pero no los de tareas con hora.
- [ ] Vista de semana: las líneas entre días coinciden con la cabecera fija en todas las columnas, con y sin scroll, y al redimensionar la ventana.
- [ ] Tema claro (y «Sistema» con Windows en claro): fondo claro opaco, texto y bordes con buen contraste, sin tono oscuro en esquinas ni bordes; Ajustes explica que la ventana es opaca.
- [ ] Cambiar entre claro y oscuro en Ajustes, y el tema de Windows con «Sistema», aplica Mica en oscuro y fondo opaco en claro sin parpadeos.

## Versión 1.0.2

- [ ] En el detalle de una tarea, el botón «☑» de la barra de notas crea una lista de casillas; Intro añade otra casilla y Intro en una vacía sale de la lista.
- [ ] Un clic en una casilla la marca o desmarca sin entrar a editar el texto; se tacha al marcarla y el cambio persiste al cerrar y reabrir la tarea.
- [ ] Las casillas se ven bien (tamaño, alineación, color de acento) en tema claro y oscuro.

## Versión 1.0.3

- [ ] En el diálogo de tarea, escribir `23112026` (o `23 11 2026`) en el campo de fecha DD/MM/AAAA deja `23/11/2026`, sin saltar al año 0002 ni perder dígitos.
- [ ] Lo mismo en la fecha de la fila de la lista: no se mueve la tarea hasta completar el año.
- [ ] Borrar la fecha sigue funcionando.

---

## Registro de sesiones de prueba

| Fecha | Build / commit | Equipo | Fase | Resultado | Responsable |
|-------|----------------|--------|------|-----------|-------------|
| | | | | | |
