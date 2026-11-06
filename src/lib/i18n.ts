export const t = {
  app: {
    name: "MiAgendaApp",
  },
  window: {
    minimize: "Minimizar",
    toggleMaximize: "Maximizar o restaurar",
    close: "Cerrar",
  },
  tasks: {
    sectionTodo: "POR HACER",
    sectionAgenda: "AGENDA",
    completed: "Completadas",
    newTaskPlaceholder: "Nueva tarea",
    titleLabel: "Título de la tarea",
    newTaskLabel: "Título de la nueva tarea",
    markDone: (title: string) => `Completar «${title}»`,
    markOpen: (title: string) => `Reabrir «${title}»`,
    deleted: "Tarea eliminada",
    undo: "Deshacer",
    dismiss: "Cerrar aviso",
    pendingCount: (count: number) =>
      count === 1 ? "1 pendiente" : `${count} pendientes`,
    loading: "Cargando tareas…",
  },
  empty: {
    title: "Todo en orden",
    description: "No tienes tareas. Escribe la primera y pulsa Intro.",
    action: "Nueva tarea",
    shortcut: "Ctrl+N",
  },
  errors: {
    NOT_FOUND: "La tarea ya no existe.",
    VALIDATION: "Los datos no son válidos",
    CONFLICT: "Ya existe un elemento igual.",
    DB: "No se pudo acceder a la base de datos.",
    IO: "No se pudo leer o escribir el archivo.",
    NOTIFY: "No se pudo mostrar la notificación.",
    INTERNAL: "Ha ocurrido un error inesperado.",
  },
} as const;
