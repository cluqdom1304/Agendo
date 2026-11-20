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
- [ ] Copia de seguridad: el cuadro «Guardar como» propone `Agendo-AAAA-MM-DD.sqlite` y sobrescribir pide confirmación de Windows.
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

## Versión 1.0.4

- [ ] Una tarea con notas muestra sus notas desplegadas al abrir la app, también tras cerrarla y volver a abrirla.
- [ ] Al ocultar las notas de una tarea con el icono, siguen ocultas tras reiniciar; al volver a pulsarlo se muestran.
- [ ] Una tarea sin notas no despliega nada; pulsar su icono abre el editor vacío para escribir.
- [ ] Con más de una tarea con notas aparece «Ocultar todas las notas» / «Mostrar todas las notas» y funciona.
- [ ] Completar una tarea, cambiarle la fecha o editarla desde el diálogo no pierde lo escrito en sus notas.
- [ ] Elegir una tarea desde la búsqueda (Ctrl+K) despliega sus notas aunque estuvieran ocultas.

---

## Rediseño de interfaz (D-39)

Ventana y barra
- [ ] La ventana se arrastra desde el hueco libre de la barra de herramientas y desde la cabecera «Etiquetas» de la barra lateral; el doble clic maximiza y restaura.
- [ ] Los botones de la barra (etiquetas, «Nueva tarea», Lista/Calendario, ajustes) responden al clic y **no** arrastran la ventana.
- [ ] Minimizar, maximizar y cerrar funcionan arriba a la derecha, también con un diálogo abierto, y no tapan el botón de ajustes.
- [ ] Con la ventana al ancho mínimo (720 px) y la barra lateral abierta, los controles de la barra no se solapan.

Aspecto
- [ ] Tema oscuro con Mica/Acrílico y tema claro opaco: textos legibles en lista, barra lateral, diálogos, menús y calendario.
- [ ] Los títulos de sección se leen «Por hacer» y «Agenda» en grande; el botón de ocultar/mostrar notas queda en la línea del primer título sin taparlo.
- [ ] Diálogos (nueva tarea, detalle, ajustes) y búsqueda (Ctrl+K): superficie opaca, esquinas redondeadas, sombra; los campos muestran un anillo de acento al enfocarlos.
- [ ] Ajustes: cada opción en su tarjeta; el interruptor «Iniciar con Windows» se ve y cambia de estado.
- [ ] Barra lateral: el punto de cada etiqueta está relleno si se muestra y hueco si está oculta.
- [ ] Calendario (día, semana, mes) dentro de su tarjeta: las columnas siguen alineadas con la cabecera y el día de hoy se distingue.

Movimiento y fluidez
- [ ] El control segmentado desliza su pastilla al cambiar de opción (Lista/Calendario, Día/Semana/Mes, tema).
- [ ] Marcar una tarea: la casilla rebota y la marca se dibuja; la fila se desplaza a «Completadas» como antes.
- [ ] Botones e iconos se encogen levemente al pulsarlos; el hover aparece sin retraso.
- [ ] Abrir diálogos, menú de etiquetas, recordatorios y avisos se siente inmediato, sin tirones; el aviso de «deshacer» aparece centrado abajo.
- [ ] Abrir y cerrar la barra lateral con muchas tareas en la lista no da tirones.
- [ ] Arrastrar una tarea hacia una etiqueta: el fantasma sigue al puntero con suavidad.
- [ ] Con «reducir movimiento» activado en Windows no hay animaciones.
- [ ] La RAM en reposo y el arranque no empeoran respecto a la 1.0.4 (mismo método de medida que N-01/N-02).

---

## Notas en la lista (D-40)

- [ ] Al abrir la app con tareas que tienen notas, el cursor **no** está en ninguna nota y no se ve ninguna barra de formato.
- [ ] Ya no existe el botón «Ocultar/Mostrar todas las notas».
- [ ] Las notas se leen como una segunda línea gris bajo el título, alineada con su texto, sin borde lateral.
- [ ] Al hacer clic en una nota pasa a texto normal con fondo de campo y la barra de formato aparece justo debajo del título de la tarea, encima del texto; al salir desaparece.
- [ ] Todas las tareas con notas las muestran desplegadas al abrir la app, también las que se ocultaron en la sesión anterior (sustituye al segundo ítem de «Versión 1.0.4»).
- [ ] El icono de notas de una tarea sin notas abre el editor con el cursor dentro y el texto «Añadir nota…»; si se sale sin escribir, se recoge solo.
- [ ] El icono de notas de una tarea con notas las oculta y las vuelve a mostrar (con el cursor dentro al mostrarlas).
- [ ] El campo de enlace (Ctrl+L) aparece bajo la barra de formato y funciona.
- [ ] Las casillas de las notas son redondas, se marcan con un clic y el texto queda tachado.
- [ ] En el detalle de una tarea (desde el calendario) las notas son un campo con anillo de acento al enfocarlo.

---

## Registro de sesiones de prueba

| Fecha | Build / commit | Equipo | Fase | Resultado | Responsable |
|-------|----------------|--------|------|-----------|-------------|
| | | | | | |

## Versión 1.0.6
- [H] La etiqueta de cada tarea se ve a la izquierda, entre la casilla y el título; las tareas sin etiqueta no dejan hueco y su botón de asignar aparece al pasar el ratón.
- [H] Ejecutar `MiAgendaApp_1.0.6_x64-setup.exe` con la 1.0.5 instalada: no pide confirmación, desinstala la anterior, instala la nueva y conserva tareas y ajustes.

## Cambio de nombre a Agendo (D-42)

- [H] Con MiAgendaApp instalada: Ajustes → «Crear copia…» y guardar el `.sqlite`; después desinstalarla.
- [H] Instalar Agendo: el título de ventana, el acceso directo, «Aplicaciones instaladas» y el ejecutable muestran «Agendo»; arranca con la lista vacía.
- [H] Ajustes → «Restaurar copia…» con el `.sqlite` guardado: reaparecen todas las tareas, etiquetas, notas y recordatorios.
- [H] La base nueva está en `%APPDATA%\com.cristian.agendo\agenda.sqlite`.

## Notas V2 — Fases 8, 9 y 10 (`[H]`, pendientes de validar en Windows 11)
- [ ] Pulsar «Notas» en la barra superior (o `Ctrl+Mayús+2`) cambia de sección con una transición suave; `Ctrl+Mayús+1` vuelve a la Agenda. La sección, carpeta y nota abiertas se recuerdan al reiniciar.
- [ ] Crear carpeta, subcarpeta, renombrar y eliminar (con confirmación): los contadores y el árbol se actualizan; no deja crear más de 5 niveles ni dos hermanas con el mismo nombre.
- [ ] Arrastrar una nota o una carpeta sobre una carpeta, la Bandeja o «Carpetas» la mueve; Esc cancela el arrastre.
- [ ] Escribir en una nota: título y cuerpo se guardan solos; al cambiar de nota no se pierde nada; la barra de formato siempre visible en la cabecera.
- [ ] Fijar, mover (menú de carpeta), duplicar y eliminar (menú «···») con «Deshacer», y restaurar desde la Papelera.
- [ ] Lista con unas miles de notas: el desplazamiento es fluido; flechas arriba/abajo cambian de nota.
- [ ] Aspecto: tema claro y oscuro, animaciones suaves y sin parpadeos, foco visible con teclado y lector de pantalla.
- [ ] (F9) Asignar varias etiquetas a una tarea desde el menú de la fila y desde el diálogo; la fila muestra dos y `+N`; filtrar por una etiqueta en la barra muestra las tareas que la tengan aunque tengan otras.
- [ ] (F9) En una nota: asignar etiquetas con el botón y con `#` (lista bajo el cursor; flechas, Intro, Tab, Esc); la etiqueta aparece en la lista de notas.
- [ ] (F9) En «Notas», la sección Etiquetas muestra contadores; al elegir una, las pestañas Notas/Tareas cambian de contenido y pulsar una tarea abre la agenda.
- [ ] (F9) Editar una etiqueta: ponerla dentro de otra, fusionarla con confirmación y comprobar que tareas y notas pasan a la etiqueta destino.
- [ ] (F10) En una nota, escribir `/tarea` y espacio (o `Ctrl+Mayús+T`, o el botón +): crear una tarea con fecha, hora y aviso; aparece como fila con casilla en la nota y en la agenda, con su nota de origen.
- [ ] (F10) Marcar la casilla en la nota completa la tarea en la agenda (y al revés); renombrar o cambiar la fecha se ve en la nota; borrar la tarea deja la línea tachada con «Restaurar», que la devuelve.
- [ ] (F10) «Convertir casillas en tareas» sobre una lista con casillas marcadas y sin marcar: se crean las tareas (las marcadas, hechas) y la lista pasa a ser referencias. El panel «Tareas de esta nota» muestra el progreso.
- [ ] (F10) En el detalle de una tarea: «Crear nota para esta tarea» y «Convertir en nota» (con Deshacer); el chip de origen de la fila y del detalle abre la nota.
- [ ] (F10) Aspecto: las filas de tarea dentro de la nota en tema claro y oscuro, animación de la casilla y del progreso.

## Repaso de diseño de Notas (`[H]`, pendiente de validar en Windows 11)
- [ ] Cabecera de la nota: los iconos de formato se ven nítidos y alineados en claro y oscuro, sin franja de otro color al desplazar el texto (con y sin Mica); al estrechar la ventana desaparecen deshacer/rehacer y nada se monta sobre los botones de la derecha.
- [ ] Menús «Ordenar por», «Mover a…» (con subcarpetas sangradas) y «···»: se abren bajo su botón sin recortarse, se cierran con Esc, con clic fuera y al elegir; flechas arriba/abajo recorren las opciones. Solo hay uno abierto a la vez (también con el de etiquetas).
- [ ] Enlace (`Ctrl+L` o el botón): el campo aparece flotando bajo la barra, se cierra al aplicar, con Esc o al pulsar en el texto.
- [ ] Papelera: pulsar una nota la muestra completa y no deja escribir ni en el título ni en el texto; «Restaurar» (en la fila o en la cabecera) la devuelve y, si estaba abierta, pasa a ser editable. Con la papelera a la vista, una nota normal abierta antes sigue editable.
- [ ] Arrastre: una nota sobre «Carpetas» y una carpeta sobre la Bandeja no se iluminan ni dan error.
- [ ] Lista: la nota abierta se distingue bien con el tinte de acento en claro y oscuro; las filas con etiquetas no se cortan; sin etiquetas el extracto ocupa dos líneas. Pasar el ratón por una etiqueta de la barra no oculta su contador.
- [ ] Nota larga y nota vacía: el texto se lee en una columna centrada en ventanas anchas; pulsar bajo el último párrafo coloca el cursor; «Tareas de esta nota» solo aparece cuando hay tareas.
- [ ] Barra de formato de la descripción de una tarea (agenda): mismos iconos nuevos, sin cambios de comportamiento.

## Notas V2 — Fase 11: enlaces, búsqueda y navegación rápida (`[H]`, pendientes de validar en Windows 11)
- [ ] (F11-01) En una nota, escribir `[[` abre una lista bajo el cursor con las notas que coinciden; flechas, Intro/Tab o clic enlazan; Esc la oculta. Si no hay una nota con ese título aparece «Crear la nota «…»», que la crea sin salir de la nota actual y deja el enlace puesto.
- [ ] (F11-01) El enlace se ve en color de acento y subrayado suave; pulsarlo abre la nota. Renombrar la nota enlazada cambia el texto del enlace en cuanto se vuelve a la nota; si se envía a la papelera el enlace sale tachado como «Nota eliminada» y se recupera al restaurarla.
- [ ] (F11-02) Una nota enlazada desde otras muestra al pie «Enlaces a esta nota» con las notas de origen (plegable, un clic abre la nota). Sin enlaces entrantes no aparece nada.
- [ ] (F11-02) Carpeta inteligente «Notas sin enlaces» en la barra lateral: lista las notas que ni enlazan ni reciben enlaces; su estado vacío es legible.
- [ ] (F11-03) `Ctrl+K` busca a la vez en notas, tareas y etiquetas, con grupos y contadores. Probar `tag:trabajo`, `in:clase`, `is:tarea`, `is:hecha`, `is:fijada` y combinaciones (`informe tag:trabajo is:pendiente`); valores con espacios entre comillas.
- [ ] (F11-04) `Ctrl+K` funciona igual en la Agenda y en Notas; `>` muestra solo comandos (nueva tarea/nota, cambiar vista o sección, carpetas inteligentes). Elegir una tarea la revela en la lista; elegir una etiqueta filtra la agenda o abre la etiqueta en Notas.
- [ ] (F11-05) `Ctrl+P` («Ir a…») lista las notas abiertas hace poco y, al escribir, filtra por título; Intro abre la nota. Tras cerrar y abrir la aplicación la lista de recientes se conserva. `Ctrl+P` no abre el diálogo de impresión del navegador.
- [ ] (F11) Aspecto: la paleta (grupos, fila activa en acento, atajos a la derecha, animación de entrada) y la lista de enlaces en tema claro y oscuro, con y sin Mica.

## Notas V2 — Fase 11B: menús contextuales y botón flotante (`[H]`, pendientes de validar en Windows 11)
- [ ] (F11B-01) Clic derecho sobre una tarea (lista y calendario) abre el menú junto al puntero, sin salirse de la ventana ni siquiera en los bordes ni en la esquina inferior derecha; no aparece el menú del navegador (ni «Inspeccionar»). Con el foco en una fila, la tecla Menú o Mayús+F10 lo abre; flechas, → / ←, Esc y clic fuera funcionan, y el foco vuelve a la fila.
- [ ] (F11B-01) Dentro de un cuadro de texto (título, búsqueda, fecha) el clic derecho sigue mostrando el menú normal de Windows (copiar, pegar, ortografía).
- [ ] (F11B-02) Tarea: marcar hecha / reabrir, fecha (hoy, mañana, elegir, quitar), aviso (abre el panel bajo la fila en la lista; en el calendario, el diálogo), etiquetas con varias casillas sin cerrar el menú, duplicar, crear nota para la tarea, convertir en nota y borrar (con el aviso Deshacer).
- [ ] (F11B-03) Nota: fijar, mover a una carpeta o a la bandeja, etiquetas, duplicar, borrar; en la Papelera solo abrir y restaurar. Carpeta: nueva nota aquí, subcarpeta, renombrar, eliminar (pide confirmación). Etiqueta en Agenda: mostrar solo esta, editar, fusionar; en Notas: ver sus notas, nueva nota con la etiqueta, fusionar.
- [ ] (F11B-04) Fondo de la Agenda: «Nueva tarea el dd/mm/aaaa» sobre un día del calendario (el diálogo se abre en ese día), ir a ese día, ir a hoy y cambiar de vista. Fondo de Notas: «Nueva nota». En el editor: cortar, copiar, pegar y seleccionar todo; sobre un enlace a nota, abrir y quitar; sobre una referencia de tarea, abrir, marcar hecha y quitar.
- [ ] (F11B-05) El botón redondo «Nueva tarea» está abajo a la derecha en Lista y Calendario, no tapa filas ni el aviso de Deshacer, no aparece en Notas, y la barra superior ya no lo tiene. Probar en ventana estrecha, con tema claro/oscuro y con `Ctrl+N` (que sigue creando en línea).
- [ ] (F11B) Aspecto: el menú (fila activa en acento, separadores, flecha de submenú, animación de entrada) en tema claro y oscuro, con y sin Mica; submenús que se abren a la izquierda cuando no caben a la derecha.

## Notas V2 — Fase 12: editor avanzado y adjuntos (`[H]`, pendientes de validar en Windows 11)
- [ ] (F12-02) Pegar una captura (Ctrl+V), arrastrar un fichero PNG/JPEG desde el Explorador y elegirlo con el botón «Insertar imagen» ponen la imagen en la nota. Pegar la misma imagen dos veces no duplica el fichero en `attachments/` (carpeta de datos de la aplicación).
- [ ] (F12-02) Arrastrar la esquina de la imagen (o enfocarla con Tab y usar las flechas) cambia su ancho y se conserva al cerrar y abrir la nota; clic en la imagen la abre grande, con Esc, la X o clic fuera para cerrar y el foco de vuelta en la nota. Un SVG, un `.exe` o un fichero de más de 20 MB se rechazan con el aviso «No se pudo guardar la imagen.».
- [ ] (F12-02) Borrar la imagen de la nota y esperar más de 24 h (o reiniciar tras ese tiempo) elimina el fichero de `attachments/`; mientras la imagen siga en alguna nota, o en la papelera, el fichero se conserva.
- [ ] (F12-03) Barra de formato de la nota: título (cada pulsación cambia de nivel y la tercera lo quita), cita, bloque de código, resaltado, línea separadora, tabla 3×3 con fila de encabezado. Clic derecho en una celda ofrece insertar/eliminar filas y columnas, fila de encabezado y eliminar la tabla; Tab salta de celda en celda.
- [ ] (F12-03) Bloque de código: comentarios, cadenas, números y palabras clave en colores legibles en tema claro y oscuro; un bloque con lenguaje (```js, ```python, ```rust, ```sql) usa el suyo.
- [ ] (F12-04) Atajos Markdown al escribir: `# `, `## `, `### ` (títulos), `> ` (cita), ``` ``` ``` (bloque de código), `---` (separador), `- ` / `1. ` / `[ ] ` (listas), `**negrita**`, `*cursiva*`, `` `código` ``, `==resaltado==`.
- [ ] (F12-04) `Ctrl+F` dentro de una nota abre la búsqueda en la propia nota (no la del navegador): marca todas las coincidencias sin distinguir mayúsculas ni acentos, Intro / Mayús+Intro recorren y dan la vuelta, el contador dice «n de m», Esc la cierra y devuelve el foco al texto. El pie de la nota muestra palabras y caracteres.
- [ ] (F12-05) «Crear copia…» propone `Agendo-AAAA-MM-DD.agendo`; en un equipo sin datos, «Restaurar copia…» con ese fichero recupera tareas, notas y sus imágenes. Una copia `.sqlite` de una versión anterior sigue restaurándose. Un `.agendo` dañado o renombrado se rechaza sin cambiar nada.
- [ ] (F12) Aspecto: tablas, citas, código, resaltado, búsqueda y vista previa de imagen en tema claro y oscuro, con y sin Mica, y con una ventana estrecha (la barra de formato se desplaza en horizontal).


## Notas V2 — Fase 13: productividad (`[H]`, pendientes de validar en Windows 11)
- [ ] (F13-01) «Nota del día» en la cabecera del calendario abre (y la primera vez crea) la nota del día mostrado, titulada «Nota del DD/MM/AAAA»; pulsarlo otra vez, o desde otro día y volviendo, abre la misma. Si se envió a la papelera, vuelve de ella. El comando «Abrir la nota de hoy» de la paleta (`Ctrl+K`) hace lo mismo con la fecha de hoy.
- [ ] (F13-02) «Plantillas de nota…» (clic derecho en el fondo de la lista de notas, o `Ctrl+K`) abre el diálogo: con una nota abierta, guardarla con un nombre la añade; «Usar» crea una nota con ese contenido y el nombre como título; «Renombrar», «Reemplazar con la nota abierta» y «Eliminar» funcionan; un nombre repetido (aunque cambien mayúsculas) muestra el aviso. Esc cierra y devuelve el foco. Revisar el aspecto en tema claro y oscuro y con la ventana estrecha.
- [ ] (F13-03) Editar una nota, esperar más de 10 minutos y volver a editar: «Más» → «Historial de versiones…» lista la versión anterior. Elegirla muestra las diferencias con la nota actual (líneas añadidas con «+», solo-en-la-versión con «−» y tachadas, legibles en tema claro y oscuro); «Restaurar esta versión» recompone la nota con ese texto y el aviso lo confirma; el texto reemplazado aparece como una versión nueva. Una nota nueva sin texto no tiene versiones. Escribir justo antes de abrir el historial no pierde lo tecleado.
- [ ] (F13-04) «Más» → «Recordar esta nota…» fija fecha y hora (por defecto mañana 09:00; una fecha pasada deshabilita «Guardar recordatorio»). A la hora, sale una notificación de Windows con el título de la nota, sus etiquetas y el inicio del texto, y en Notas aparece el aviso «Recordatorio de la nota «…»». Volver a abrir el diálogo muestra la hora fijada y permite quitarla; una nota enviada a la papelera no avisa; con la aplicación cerrada a la bandeja, también avisa. Comprobar que los avisos de tareas siguen sonando igual.
- [ ] (F13-06) Una tarea con aviso: la notificación de Windows muestra el botón «Marcar como hecha»; pulsarlo (con la ventana minimizada, en bandeja o a la vista) completa la tarea y la lista se actualiza sola al volver. Hacer clic en el cuerpo de la notificación trae la ventana al frente y abre esa tarea (o la nota, en un aviso de nota). Si el toast ya pasó al Centro de actividades puede que no responda (límite de Windows/plugin): anotarlo. Un aviso de nota no muestra el botón. Si Windows no informa de qué notificación es, no debe completarse ninguna tarea.
- [ ] (F13-07) `Ctrl+Mayús+3` (o «Hoy» en el selector de sección) muestra la fecha de hoy con sus tareas: las abiertas primero por hora, las hechas al final y tachadas; la casilla completa/reabre y el clic abre el detalle. «Abrir la nota de hoy» salta a Notas con la nota diaria. Las notas fijadas aparecen y se abren con clic. Al reiniciar, la aplicación vuelve a «Hoy». Dejar la ventana abierta pasada la medianoche y volver a ella: muestra el día nuevo. Aspecto en tema claro y oscuro y con la ventana estrecha.
- [ ] (F13-05) En «Nueva tarea» o al editar una con fecha: «Repetir la tarea» (cada día/semana/mes/año), «Cada N» y «Hasta» opcional; sin fecha el selector está desactivado. Al completar una tarea repetida aparece la siguiente (con la misma hora, etiquetas y avisos relativos) y la lista se actualiza sola; completarla con retraso salta a la primera fecha futura de su ciclo. Una tarea mensual del 31 pasa al 28/29 en febrero y vuelve al 31 en marzo. El detalle muestra «Se repite: …». Quitar la fecha o llevarla a «Por hacer» deja de repetirla. Reabrir y volver a completar la tarea no crea una copia más.
- [ ] (F13) Diálogos sin desplazamiento horizontal: pulsar una tarea creada desde una nota (detalle de tarea), «Nueva tarea» desde la nota (`/tarea`), «Recordar esta nota…» y «Plantillas de nota…» se ven enteros, apilados en vertical, también con la ventana estrecha y al 125–150 % de escala de Windows; los botones del detalle pasan a una segunda línea si no caben.


## Notas V2 — Fase 14: interoperabilidad (`[H]`, pendientes de validar en Windows 11)
- [ ] (F14-01) «Más» → «Exportar a Markdown…» / «Exportar a HTML…» en una nota, y clic derecho en una carpeta → «Exportar a Markdown…» / «HTML…», piden una carpeta de destino y escriben los archivos (la nota suelta directamente; la carpeta con sus subcarpetas). El `.md` se abre bien en otro programa (Obsidian, VS Code): front matter, título, listas, casillas, tablas, citas, código e imágenes (carpeta `adjuntos/` al lado). El `.html` se abre en el navegador sin conexión, con las imágenes incluidas, en tema claro. Títulos con caracteres como `/ : ?` o repetidos dan archivos distintos y válidos. El aviso dice cuántas notas e imágenes se exportaron.
- [ ] (F14-01) Escribir algo justo antes de exportar la nota abierta: el archivo lo incluye. Exportar a una carpeta sin permiso de escritura muestra el error, sin cerrar la aplicación.
- [ ] (F14-02) Ajustes → «Exportar todo a JSON…» propone `Agendo-AAAA-MM-DD.json`; el archivo es JSON válido con carpetas, etiquetas, notas, tareas con avisos y plantillas. Elegir como destino el `.sqlite` de la base de datos se rechaza.
- [ ] (F14-01) Ajustes → «Exportar notas a Markdown…» / «HTML…» exporta todas las notas (la Bandeja en la raíz, cada carpeta como carpeta) y el aviso cuenta notas e imágenes.
- [ ] (F14-03) Ajustes → «Importar carpeta de Markdown o texto…» con una carpeta que tenga subcarpetas, `.md` con y sin front matter, `.txt` e imágenes relativas: crea las carpetas (hasta 5 niveles), las notas con su título (front matter, primer `# H1` o nombre del archivo), sus etiquetas y fechas, y las imágenes. Una carpeta exportada por Agendo se importa de vuelta igual. Las vistas de Notas y Etiquetas se actualizan solas. Una imagen apuntada fuera de la carpeta o con `http://` no se importa y se cuenta como omitida.
- [ ] (F14-04) Ajustes → «Importar desde Evernote…» con uno o varios `.enex` reales (idealmente con imágenes, listas, casillas, tablas y etiquetas): cada archivo es una carpeta con su nombre; las notas conservan título, fechas de creación y modificación, etiquetas, formato, casillas e imágenes. Los PDF adjuntos no se importan y se cuentan como omitidos. Un archivo que no sea `.enex` se rechaza con un mensaje claro. Probar con un cuaderno grande (miles de notas): la ventana sigue viva y avisa al terminar.
- [ ] (F14) Aspecto: la sección «Exportar e importar» de Ajustes se ve entera, con botones que saltan de línea y sin desplazamiento horizontal, también con la ventana estrecha, en tema claro y oscuro.

## Repaso de diseño previo a la 2.0 (D-55) (`[H]`, pendiente de validar en Windows 11)
- [ ] «Hoy»: fecha sobre el título grande; tareas, nota del día y notas fijadas en tarjetas con filas separadas por una línea fina; la fila entera se ilumina al pasar el puntero; con el día vacío el aviso queda dentro de su tarjeta. Se ve bien en claro y oscuro y con la ventana estrecha.
- [ ] Menús de «Ordenar», «Mover a», «Más» y etiquetas de la nota: la entrada bajo el puntero se pinta en color de acento (en rojo «Eliminar»), con el texto y la marca legibles; con las flechas del teclado ocurre lo mismo.
- [ ] «Más» → «Historial de versiones»: el diálogo es ancho, sin contenido recortado ni desplazamiento horizontal; la versión elegida queda teñida de acento; líneas añadidas en verde y quitadas en rojo tachado, legibles en ambos temas.
- [ ] «Plantillas de nota»: lista en una tarjeta; «Usar» destaca y el resto de acciones son discretas; los botones saltan de línea sin desbordar con nombres largos.
- [ ] En una nota: bloque de código con colores legibles también con el tema forzado al contrario del sistema (Ajustes → Tema); resaltado y coincidencias de Ctrl+F en amarillo; cita, regla y bordes de tabla visibles en claro y oscuro.
- [ ] Ctrl+F y el campo de enlace (Ctrl+L) muestran el mismo anillo de foco que los campos de los diálogos. «Enlaces entrantes» y «Tareas de esta nota» iluminan la fila al pasar el puntero y recortan con puntos suspensivos los textos largos.
- [ ] Seleccionar texto en cualquier parte usa un tinte del color de acento. El aviso inferior («Nota eliminada · Deshacer») muestra un aspa nítida para cerrar.

## Cierre de 2.0.0 — Fase 15 (`[H]`, pendientes de validar en Windows 11)
- [ ] (F15-02) Instalar la 2.0.0 sobre un equipo con la 1.0.7 y datos reales: la aplicación abre sin error, conserva tareas, etiquetas, avisos y ajustes, y la sección Notas empieza vacía. Hacer antes una copia `.sqlite` desde la 1.0.7 por si acaso.
- [ ] (F15-01) Con una base de 10 000 notas (por ejemplo, importando una carpeta de Markdown grande) el arranque, el cambio entre carpetas y la búsqueda (`Ctrl+K`) se sienten inmediatos, sin parpadeos al desplazar la lista.
- [ ] (F15-03) Ejecutar `npm run e2e` (ver `e2e/README.md`): las tres especificaciones (`tareas`, `ajustes`, `notas`) pasan. Es la primera ejecución real de F6-06; anotar cualquier selector que falle.
- [ ] (F15-04) El instalador `Agendo_2.0.0_x64-setup.exe` muestra la versión 2.0.0 en «Aplicaciones instaladas», reemplaza la 1.0.7 sin pedir confirmación y la ventana sigue mostrando «Agendo».
- [ ] (Todas las fases V2) Recorrer las secciones «Notas V2 — Fase 8» a «Fase 14» de este archivo y marcar lo comprobado.

## Versión 2.0.1 (`[H]`)
- [ ] Clic derecho en una tarea, una nota, una carpeta y una etiqueta pegado al borde superior, inferior, izquierdo y derecho y en las esquinas: el menú entero queda a la vista, sin cortarse. Abrir un submenú («Mover a») cerca del borde inferior y del superior: se desliza para caber y se desplaza si hay muchas carpetas. Con la ventana al tamaño mínimo (720×480) ocurre lo mismo.

## Versión 2.0.2 (`[H]`)
- [ ] Ctrl+N en la agenda (lista, día, semana y mes) abre el diálogo «Nueva tarea» con el título enfocado; con un diálogo ya abierto no abre otro; Ctrl+Mayús+N sigue creando un borrador en línea de hoy. En «Notas» Ctrl+N sigue creando una nota.
- [ ] Escribir una palabra mal en la descripción de una tarea y en el editor de una nota: el subrayado rojo aparece al escribir y desaparece al quitar el foco; vuelve al recuperarlo.
- [ ] Clic derecho en una tarea pegada a la barra lateral › «Etiquetas»: el submenú se ve entero, sin cortarse con la barra izquierda ni con el borde derecho.
- [ ] Ajustes › «Acerca de» › «Información de la aplicación»: muestra versión 2.0.2, sistema, WebView2, Tauri, SQLite, esquema (14), carpeta de datos (la `.dev` con `tauri dev`), tamaño y recuentos coherentes con lo que hay.
- [ ] Escribir en una nota y pulsar enseguida el botón de cerrar de la ventana: al reabrir, el último texto está.

## Versión 2.0.3 (`[H]`)
- [ ] En la lista, pulsar el icono de etiqueta de una tarea (con la barra lateral abierta y también con la ventana al tamaño mínimo): el menú se ve entero, sin cortarse con la barra izquierda ni con el borde derecho; en una tarea del final de la lista se abre hacia arriba. Marcar varias etiquetas seguidas lo mantiene abierto; desplazar la lista o cambiar el tamaño de la ventana lo cierra. Lo mismo en el editor de notas.

## Versión 2.1.0 (`[H]`, pendientes; se completan al cerrar cada fase)

Fase 16 — rendimiento (F16-09)
- [x] Medir en Windows 11 el arranque (N-01, < 200 ms) y la memoria en reposo (N-02, < 40 MB) con el script de F16-06 y anotarlos en `docs/metricas.md`.
- [x] La agenda y la lista de notas se desplazan sin tirones con datos reales (miles de tareas y notas).
- [x] La primera apertura del calendario, de Ajustes y del historial de versiones no muestra parpadeos ni esperas perceptibles (carga diferida).
- [x] Ejecutar `npm run e2e` con `tauri-driver` (F6-06 y F15-03, nunca ejecutados) y anotar el resultado.

Fase 17 — mejoras de lo existente (F17-10)
- [ ] F17-01: crear una tarea semanal con fecha un lunes, marcar lunes y miércoles y «Número de veces» = 3; completarla tres veces y comprobar que aparecen miércoles y lunes y que a la tercera no nace otra. Revisar el aspecto de los botones de día en claro y oscuro.
- [ ] F17-02: con un recordatorio de una tarea, comprobar que la notificación muestra «Marcar como hecha» y los tres «Posponer»; pulsar «Posponer 10 min» y verificar que vuelve a sonar a los 10 minutos y que la tarea sigue abierta. Si Windows recorta los botones, anotar cuáles.
- [ ] F17-03: Ctrl+clic en tres tareas y Mayús+clic para un rango; comprobar el tinte de selección, la barra inferior (no tapa el aviso de deshacer) y que Completar, Etiquetar, Mover al día y Eliminar actúan sobre todas; un solo «Deshacer» (botón y Ctrl+Z) lo revierte; Esc quita la selección.
- [ ] F17-04: en una nota, tachar y subrayar texto con los botones y con Ctrl+U / Ctrl+Mayús+S; exportar a Markdown y a HTML y abrir los archivos en otro programa (VS Code, navegador) comprobando que se ven; importar ese `.md` de vuelta.
- [ ] F17-06: importar dos veces la misma carpeta de Markdown y un `.enex` real; la segunda vez el aviso debe decir que las notas ya estaban importadas y no deben aparecer copias. Editar un archivo y reimportar: entra como nota nueva.
- [ ] F17-07: crear una etiqueta «SMA» con dos subetiquetas; comprobar que desmarcar «SMA» oculta las tareas de las hijas, que «Solo» en «SMA» muestra las de sus hijas y que el contador del padre suma las de las hijas.
- [ ] F17-08: en Ctrl+K escribir `is:pendiente tag:SMA` y «Guardar esta búsqueda»; reabrir la paleta con la caja vacía, comprobar que aparece en «Búsquedas guardadas», que al elegirla busca y que la ✕ la quita. Probar `informe desde:2026-03-01 hasta:2026-03-31` y comprobar que respeta el rango.
- [ ] F17-09: en Ajustes elegir «Cada día» y conservar 3; dejar la aplicación abierta (la primera copia llega a los ~45 s), comprobar que aparece `agendo-auto-….agendo` en la carpeta `backups` junto a la base de datos y el texto «Última copia automática». Restaurar esa copia desde Ajustes. Con el reloj del sistema adelantado varios días, comprobar que solo quedan 3.
- [ ] _Sin más ítems: F17-02 a F17-09 están detallados arriba._

Fase 18 — resumen semanal (F18-04)
- [ ] _Se detallan al implementar F18-01 a F18-03._


Auditoría A1 — seguridad
- [ ] A1-01: con la versión release (`npm run tauri -- build`), abrir Agendo y comprobar que arranca, que se ven y editan notas con imágenes pegadas, que los diálogos, la paleta, el calendario y los ajustes se muestran con su estilo, y que abrir un enlace `https` de una nota funciona. Si algo falla por la CSP, la consola de WebView2 (F12 con una build de depuración) muestra «Refused to …»: anotar la directiva y avisar.
## Auditoría A6–A12 (`[H]`)

Todo lo de esta sección requiere Windows 11 real; ninguno está verificado.

- [ ] A6-03: en la vista Día o Semana con la hora actual visible, la línea de «ahora» avanza sola cada minuto y, pasada la medianoche, «Hoy» cambia de día sin reabrir.
- [ ] A12-01: con la base de datos inaccesible o dañada (renombrar la carpeta de datos con la aplicación cerrada), la lista de tareas muestra «No se pudieron cargar las tareas» con «Reintentar» (no «Sin tareas»), y Notas muestra «No se pudieron cargar las notas» con «Reintentar». Al restaurar la carpeta, «Reintentar» recupera los datos.
- [ ] A12-01: el estado de carga («Cargando tareas…», «Cargando calendario…») no parpadea de forma molesta al abrir; el Narrador lo anuncia una vez y el error se anuncia como alerta.
- [ ] A12-02: solo con teclado, recorrer la lista con las flechas, completar con Espacio, borrar con Supr y abrir las acciones de una fila (recordatorios, etiquetas, nota de origen) con la tecla Menú o Mayús+F10; Esc cierra la paleta, la vista previa de imagen y los diálogos, y el foco vuelve al elemento que los abrió.
- [ ] A12-03: instalar encima de una versión anterior con la aplicación abierta: el instalador cierra o avisa del proceso `agendo.exe` y no deja archivos bloqueados. Comparar `src-tauri/tauri.conf.json` (`bundle.windows.nsis`) con la plantilla NSIS de Tauri v2 y anotar el diff.
- [ ] A12-04: pasada final en Windows 11 de todas las secciones anteriores de este manual (instalación limpia y actualización, Mica y tema claro/oscuro/sistema, recordatorios con la aplicación cerrada, copia de seguridad y restauración, importación y exportación, adjuntos, atajos) con la versión que se vaya a publicar; marcar aquí solo lo que una persona haya comprobado.
- [ ] D-70 (si se acepta la opción C): revisión visual de todas las vistas tras retirar Tailwind (márgenes de encabezados, botones y campos, imágenes).

## Auditoría A8 (`[H]`)

Refactor de estructura sin cambios de comportamiento; ninguno está verificado en Windows 11.

- [ ] A8-02: atajos Ctrl+K, Ctrl+P, Ctrl+N (con y sin un diálogo abierto), Ctrl+1..3 de sección y cambio de vista de agenda; recordatorio de tarea y de nota (aviso y clic en la notificación abren su destino); paleta, menú contextual, diálogo de tarea, vista de tarea y ajustes se abren por encima del botón «+» y del aviso de deshacer.
- [ ] A8-03: en una nota, escribir `#` (lista de etiquetas con flechas, Enter, Tab y Esc) y `[[` (lista de notas, opción «crear», Esc); el menú contextual del editor, de tareas, notas, carpetas y etiquetas ofrece las mismas entradas que antes.
- [ ] A8-05: Ajustes > Acerca de: el botón carga y muestra la información con el mismo aspecto (rejilla, textos secundarios) que antes.

## Auditoría A2–A4 (`[H]`, pendientes de validar en Windows 11)
- [ ] Abrir una base de datos creada con la 2.0.x: arranca sin error, Ajustes › «Acerca de» muestra esquema (14) y la búsqueda de tareas y de notas encuentra lo mismo que antes (la migración 012 reindexa el buscador).
- [ ] Editar el texto de una nota o tarea y buscar una palabra nueva y una antigua: la nueva aparece, la antigua ya no (índice por rowid, D-62).
- [ ] Con muchas notas (1 000 o más, p. ej. importadas), la lista de notas y la agenda con recordatorios abren sin pausa notable (D-63, D-64).
- [ ] Hacer una copia `.agendo` de una base grande mientras se escribe en una nota: el guardado no se queda esperando a que acabe la copia (D-65).
- [ ] Importar un `.enex` grande (cientos de MB): la memoria de la aplicación no crece con el tamaño del archivo y las notas entran todas (D-65).
- [ ] Restaurar una copia `.agendo` con adjuntos: las imágenes vuelven y las notas las muestran.
- [ ] Cerrar la aplicación desde la bandeja con una copia automática en marcha: no queda ningún `.tmp` en `backups` y la siguiente copia se hace con normalidad (D-66).
- [ ] Exportar a Markdown varias notas que comparten una imagen: la carpeta `adjuntos` tiene una sola copia y todas las notas la enlazan.

## Auditoría A6-A11 (frontend)

Ninguno está verificado; requieren Windows 11 real.

- [ ] A11-03: primera ejecución de e2e (`npm run e2e`) con las nuevas especificaciones `calendar`, `reminders`, `tags` y `backups` de `e2e/specs/`: se escribieron sin poder ejecutarlas, así que pueden necesitar ajustes de selectores (vista Calendario como `radio`, columna `[role="gridcell"][data-date]`, campo `input[data-autofocus]` del editor de etiquetas).
- [ ] A6-01: abrir la lista con muchas tareas y provocar cambios externos seguidos (recordatorio disparado, completar una tarea repetida, importar una copia): la lista se actualiza sin parpadeos ni perder la selección ni la fila en edición.
- [ ] A8-05: Calendario > Semana y Día: arrastrar un tramo, escribir el título y Enter; Enter sobre una columna enfocada; Esc cancela. Igual que antes de dividir `CalendarWeek`.
- [ ] A6-02: abrir una tarea creada desde una nota (muestra el título de la nota de origen) y una nota con enlaces a otras notas, borradas y vivas: los enlaces borrados siguen mostrándose como eliminados.

## Auditoría — lista final `[H]` en Windows 11 (A12-04)

Orden sugerido; las secciones «Auditoría A2–A4», «A6–A12», «A8» y «A6-A11» de arriba detallan los pasos.

1. **A1-01 CSP:** instalar una build release y comprobar que editor, imágenes pegadas, calendario y ajustes funcionan sin errores de CSP (DevTools no está en release: probar con `tauri build --debug`).
2. **A12-03 Instalador:** actualizar sobre una versión instalada con Agendo abierto; debe cerrar el proceso y conservar los datos. Comparar `installer.nsi` con la plantilla de Tauri.
3. **A6-06 Editor:** abrir una nota de ~1 MB y otra con 50 imágenes; escribir y medir fluidez (Performance de WebView2).
4. **A11-03 e2e:** `npm run e2e` por primera vez (calendario, recordatorios, etiquetas, copias).
5. **A2–A4 datos:** migrar una base real a la versión 14 (hacer copia antes), comprobar búsqueda, papelera, copias y exportar/importar `.agendo` y ENEX.
6. **A5 registro:** comprobar que `agendo.log` aparece en la carpeta de datos y rota.
7. **A8 pulido visual:** revisar calendario semanal, ajustes «Acerca de» y los estados vacío/carga/error.

## U-01 — Cambio de sección sin parpadeo (`[H]`)
- [x] (confirmado por el usuario) Pasar varias veces de Agenda a Notas y a Hoy (también justo tras arrancar): la barra Agenda/Notas/Hoy no debe parpadear ni saltar, y el panel lateral debe aparecer sin desplazar la barra de herramientas.

## Versión 3.0.0 (`[H]`, pendientes; se detallan al implementar cada fase)

- **Fase 19 — shell y sistema visual (F19-06):**
  - [ ] Sidebar y barra superior oscuros en tema claro y oscuro, con Mica, Acrylic y sin efecto; texto legible en todos los casos (también en las carpetas, etiquetas y el editor de etiquetas).
  - [ ] El contenido se ve como una tarjeta redondeada con hueco de 8 px a derecha, izquierda y abajo; sin bordes cortados al maximizar ni con la ventana a 720×480.
  - [ ] La barra superior arrastra la ventana y el doble clic la maximiza; los botones no arrastran; los botones de ventana no tapan Ajustes.
  - [ ] «Buscar o crear…» y Ctrl+K abren la paleta; «nuevo» crea una tarea (o una nota en Notas).
  - [ ] Atrás/adelante recorren las secciones visitadas y se desactivan cuando no hay adónde ir.
  - [ ] Por debajo de 900 px el sidebar se pliega solo y vuelve al ensanchar; arrastrar una etiqueta o una nota lo abre.
  - [ ] Favoritos (notas fijadas) y Recientes aparecen poco después de abrir, se pliegan y abren la nota; elegir una carpeta desde Hoy, Tareas o Calendario lleva a Notas.
  - [ ] Una instalación que venía de la 2.0.3 con la agenda abierta en el calendario vuelve a abrir en «Calendario»; con la lista, en «Tareas».
- **Fase 20 — Hoy, Próximas, Bandeja y prioridades (F20-06):**
  - [ ] Hoy: el título grande, la tira de 14 días (marca bajo los días con pendientes) y el banner se ven bien en claro, oscuro y Mica; elegir un día cambia título, tareas y nota del día; las flechas, Inicio y Fin recorren la tira.
  - [ ] Con la app abierta de un día para otro, al volver a la ventana la tira empieza en el día nuevo.
  - [ ] Los cuatro niveles de prioridad se distinguen en el círculo en ambos temas; Narrador lee «…, prioridad alta».
  - [ ] Prioridad desde el diálogo de tarea y desde el menú contextual; dentro de un día, la tarea sin hora sube al ponerle prioridad.
  - [ ] Marcar casillas en las notas de una tarea actualiza el «2/5» de su fila al guardarse.
  - [ ] Bandeja de entrada en Hoy, Próximas y Calendario: añadir con Intro, completar, abrir y menú contextual; se pliega desde la barra superior y desaparece por debajo de 1100 px. (El arrastre desde la Bandeja a una franja queda para la Fase 24.)
  - [ ] Próximas: Vencidas, Hoy, Mañana, los días con nombre y «Más adelante».
  - [ ] Tareas: los filtros de fecha, prioridad mínima y «Mostrar completadas» se combinan con el filtro de etiquetas; «Quitar filtros» devuelve la lista entera.
- **Fase 21 — eventos (F21-05):**
  - [ ] Sidebar «Eventos»: «Nuevo evento» crea con título, fecha, hora de inicio y fin, etiquetas, aviso y notas; el evento sale en la lista, en Hoy, en Próximas y en el calendario.
  - [ ] Todo el día y varios días: sin horas, en la fila «Todo el día» de Semana y Día y como chip en Mes.
  - [ ] Calendario Semana y Día: un evento y una tarea con hora a la vez se reparten el ancho sin taparse; arrastrar un evento cambia su hora y de columna cambia su día; el borde inferior cambia la hora de fin; Esc a mitad de un arrastre lo cancela; un arrastre no abre el diálogo y un clic sí.
  - [ ] Evento repetido (semanal): aparece cada semana; arrastrar una ocurrencia mueve toda la serie y el diálogo avisa de que los cambios son para toda la serie.
  - [ ] Notificación nativa de un evento con aviso «15 min antes»: sale a su hora con el título; pulsarla abre la app en el diálogo del evento. No hay botones de «Hecha» ni de posponer.
  - [ ] Menú contextual: «Convertir en evento» en una tarea (la tarea va a la papelera) y «Convertir en tarea» en un evento; «Borrar» un evento ofrece «Deshacer»; clic derecho en un día del calendario ofrece «Nuevo evento el …».
  - [ ] Los eventos respetan el filtro de etiquetas del sidebar; contraste legible de los bloques de evento en claro, oscuro y Mica.
- **Fase 22 — hábitos (F22-03, validada por el usuario el 2026-11-20):**
  - [x] Sidebar «Hábitos»: «Nuevo hábito» con nombre, días, hora y etiqueta; aparece en la vista con sus 7 días.
  - [x] Marcar y desmarcar hoy y un día pasado de la semana; la racha sube y baja; un día no programado sale punteado y no rompe la racha.
  - [x] Hoy: «Hábitos de hoy» solo con los programados hoy; un clic marca, sale «Deshacer» y el banner cuenta «N de M hábitos»; no aparecen al elegir otro día de la tira.
  - [x] Con la app abierta pasada la medianoche, al volver a la ventana Hoy y Hábitos cambian de día y la racha se mantiene.
  - [x] Archivar, desarchivar y eliminar con «Deshacer»; los hábitos respetan el filtro de etiquetas.
  - [x] Contraste de las casillas marcadas, punteadas y con foco en claro, oscuro y Mica.
- **Fase 23 — lenguaje natural (sin tarea `[H]` propia en el plan; se valida con la Fase 24 o al cierre):**
  - [ ] Borrador en línea (Ctrl+Mayús+N y «Nueva tarea» de un día): al escribir «Dentista mañana a las 5 durante 1h #etiqueta !2» aparece debajo, sin tapar la fila siguiente de forma molesta, «Dentista · Mañana · 17:00–18:00 · #etiqueta · Alta · Esc: tal cual»; Intro la crea con todo eso.
  - [ ] Esc con algo detectado quita la vista previa y crea el texto tal cual; un segundo Esc descarta el borrador.
  - [ ] Calendario Semana/Día: arrastrar un hueco y escribir «Revisión a las 16:00 durante 45 min» crea a las 16:00 de ese día con 45 min; sin hora escrita, manda el hueco.
  - [ ] Diálogo «Nueva tarea»: al salir del título los campos de fecha, hora, fin, etiquetas, prioridad y repetición se rellenan y el título queda limpio; Esc en el título no cierra el diálogo si había algo detectado. Al editar una tarea existente no se interpreta nada.
  - [ ] Bandeja de entrada: «Regar pasado mañana #casa» crea la tarea con fecha (sale de la Bandeja) y la etiqueta; una `#palabra` que no es etiqueta se queda en el título.
  - [ ] La vista previa se lee bien en claro, oscuro y Mica, y Narrador anuncia «Se creará con…».
- **Fase 24 — planificar el día (F24-03):**
  - [ ] Calendario Semana y Día con la Bandeja abierta: arrastrar una tarea a una franja la deja ese día a esa hora (a cuartos de hora) con 30 min (o su duración si ya la tenía) y desaparece de la Bandeja; el bloque aparece bajo el puntero; Esc cancela; soltar fuera de una columna no cambia nada; el aviso del puntero lee «Suéltala en una franja del calendario».
  - [ ] Al arrastrar desde la Bandeja sale una etiqueta con la hora (17:00, 17:15, 17:30… al bajar) y la tarea queda a esa hora.
  - [ ] Arrastrar una tarea con hora ya puesta en Semana/Día cambia su día y su hora de 15 en 15 min con vista previa; arrastrar su borde inferior cambia la duración; un clic sin arrastrar sigue abriendo la tarea; arrastrarla hasta una etiqueta del sidebar sigue asignándola.
  - [ ] Con el calendario desplazado (scroll) la hora que se fija es la de la franja sobre la que se suelta.
  - [ ] Hoy → «Planificar mi día» (y desde Ctrl+K): vencidas marcadas, sin fecha sin marcar; la suma cambia al marcar; con más de 8 h sale el aviso en rojo; «Pasar N tareas a hoy» las mueve y Cancelar no cambia nada. Se lee bien en claro, oscuro y Mica y con teclado.
- **Fase 25 — enfoque (F25-03):**
  - [ ] Detalle de una tarea → elegir minutos y «Enfocar»: el diálogo se cierra y la barra superior muestra la cuenta atrás (mm:ss) con un punto de acento; el botón «Enfocar» queda desactivado mientras haya una sesión.
  - [ ] Pulsar la cuenta atrás de la barra superior termina la sesión: aviso «Sesión de enfoque terminada» y el tiempo aparece en la fila (p. ej. «5 min»).
  - [ ] Con una sesión corta (15 min o ajustada para probar), al llegar a cero sale la notificación nativa «Sesión de enfoque terminada», una sola vez, aunque la ventana esté minimizada; la barra superior vuelve a su estado normal.
  - [ ] Cerrar la app con el temporizador activo y abrirla de nuevo: si ya pasó el tiempo, la sesión consta cerrada a su hora prevista; si no, sigue la cuenta atrás desde donde toca.
  - [ ] Detalle de la tarea → «Tiempo dedicado»: total, lista de entradas (sesiones y manuales), «Añadir» minutos a mano y la «×» que quita una entrada; la fila refleja el total (formato «1 h 25 min») y no muestra nada con cero.
  - [ ] Lectura correcta en claro, oscuro y Mica, con teclado y Narrador.
- **Fase 26 — resumen semanal:** cifras coherentes con la agenda y exportación a Markdown.
- **Fase 27 — cierre:** abrir con una base real de la 2.0.3 y comprobar que no se pierde nada; N-01, N-02 y `npm run e2e`.

