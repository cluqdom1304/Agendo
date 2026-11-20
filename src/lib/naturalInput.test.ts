import { describe, expect, it } from "vitest";

import {
  addDays,
  firstOn,
  parseNatural,
  taskFieldsOf,
  type Capture,
} from "./naturalInput";
import type { DayCode } from "./recurrence";

// Fixed clock: Friday 20 November 2026.
const TODAY = "2026-11-20";
const TAGS: Record<string, string> = {
  trabajo: "t1",
  salud: "t2",
  "sma-1": "t3",
};
const tagId = (name: string) => TAGS[name.toLowerCase()];

const parse = (text: string, fallbackDate?: string) =>
  parseNatural(text, { today: TODAY, tagId, fallbackDate });

function expectCapture(text: string, expected: Partial<Capture>) {
  const capture = parse(text);
  expect(capture, text).not.toBeNull();
  expect(capture).toMatchObject(expected);
}

describe("parseNatural: nothing to take", () => {
  it.each([
    "Comprar pan",
    "Leer 3 libros",
    "Llamar a Juan",
    "Revisar capítulos 1-2",
    "#desconocida revisar",
    "Preparar la reunión de la mañana",
    "Correr por la mañana",
    "Ver el 2000",
  ])("%s", (text) => {
    expect(parse(text)).toBeNull();
  });

  it("keeps a lone date as the title instead of an empty task", () => {
    expect(parse("mañana")).toBeNull();
    expect(parse("#trabajo !2")).toBeNull();
  });
});

describe("parseNatural: dates", () => {
  it.each([
    ["Pagar hoy", "2026-11-20"],
    ["Pagar mañana", "2026-11-21"],
    ["Pagar pasado mañana", "2026-11-22"],
    ["Pagar en 3 días", "2026-11-23"],
    ["Pagar dentro de dos semanas", "2026-12-04"],
    ["Pagar en 1 mes", "2026-12-20"],
    ["Pagar la semana que viene", "2026-11-23"],
    ["Pagar el lunes", "2026-11-23"],
    ["Pagar el próximo miércoles", "2026-11-25"],
    ["Pagar el viernes", "2026-11-27"],
    ["Pagar este viernes", "2026-11-20"],
    ["Pagar sabado", "2026-11-21"],
    ["Pagar el 15/12", "2026-12-15"],
    ["Pagar el 3/1", "2027-01-03"],
    ["Pagar 15/12/27", "2027-12-15"],
    ["Pagar 15/12/2028", "2028-12-15"],
    ["Pagar el 15 de diciembre", "2026-12-15"],
    ["Pagar 2 dic", "2026-12-02"],
    ["Pagar el 1 de marzo de 2027", "2027-03-01"],
    ["Pagar el día 25", "2026-11-25"],
    ["Pagar el 5", "2026-12-05"],
    ["Pagar el 31", "2026-12-31"],
  ])("%s → %s", (text, date) => {
    expectCapture(text, { title: "Pagar", date });
  });

  it("rejects impossible days and keeps them in the title", () => {
    expect(parse("Pagar el 31/02")).toBeNull();
    expect(parse("Pagar el 30 de febrero de 2027")).toBeNull();
  });

  it("finds a month after a number that is not a date", () => {
    expectCapture("Leer 3 libros el 5 de mayo", {
      title: "Leer 3 libros",
      date: "2027-05-05",
    });
  });

  it("moves 29 February to the next leap year", () => {
    expectCapture("Aniversario 29/2", { date: "2028-02-29" });
  });

  it("clamps a month later to the last day of a shorter month", () => {
    expect(parseNatural("Pagar en 1 mes", { today: "2027-01-31" })?.date).toBe(
      "2027-02-28",
    );
  });
});

describe("parseNatural: times and durations", () => {
  it.each([
    ["Dentista a las 17:30", "17:30"],
    ["Dentista a las 5", "17:00"],
    ["Dentista a las 9", "09:00"],
    ["Dentista a las 5 de la mañana", "05:00"],
    ["Dentista a las 9 de la noche", "21:00"],
    ["Dentista a las 12 de la noche", "00:00"],
    ["Dentista a las 5 y media", "17:30"],
    ["Dentista a las 10 y cuarto", "10:15"],
    ["Dentista a las 17h", "17:00"],
    ["Dentista a la 1", "13:00"],
    ["Dentista 8:05", "08:05"],
    ["Dentista 6pm", "18:00"],
    ["Dentista 12am", "00:00"],
    ["Dentista 7 de la tarde", "19:00"],
    ["Dentista a mediodía", "12:00"],
  ])("%s → %s", (text, time) => {
    expectCapture(text, { title: "Dentista", time, date: TODAY });
  });

  it("rejects hours out of range", () => {
    expect(parse("Dentista a las 25")).toBeNull();
    expect(parse("Dentista 13pm")).toBeNull();
  });

  it("puts a time on the fallback day when no day is typed", () => {
    expectCapture("Llamada a las 10", { date: TODAY, time: "10:00" });
    expect(parse("Llamada a las 10", "2026-11-25")?.date).toBe("2026-11-25");
    expect(parse("Llamada mañana a las 10", "2026-11-25")?.date).toBe(
      "2026-11-21",
    );
  });

  it("does not read «de la mañana» as tomorrow", () => {
    expectCapture("Correr mañana a las 7 de la mañana", {
      title: "Correr",
      date: "2026-11-21",
      time: "07:00",
    });
  });

  it.each([
    ["Reunión a las 10 durante 1h", 60],
    ["Reunión a las 10 1h30", 90],
    ["Reunión a las 10 2 horas", 120],
    ["Reunión a las 10 1 hora y 15 minutos", 75],
    ["Reunión a las 10 por 45 min", 45],
    ["Reunión a las 10 durante media hora", 30],
    ["Reunión a las 10 hora y media", 90],
    ["Reunión a las 10 una hora", 60],
  ])("%s → %i min", (text, duration) => {
    expectCapture(text, { title: "Reunión", time: "10:00", duration });
  });

  it("takes no duration without a time", () => {
    expect(parse("Leer 30 min")).toBeNull();
  });

  it("cuts a duration that would end the next day", () => {
    expectCapture("Turno a las 23:00 durante 2h", { duration: 60 });
  });
});

describe("parseNatural: tags and priority", () => {
  it("takes known tags only, case-insensitively, without repeating them", () => {
    expectCapture("Informe #Trabajo #otra #trabajo #sma-1", {
      title: "Informe #otra",
      tags: [
        { id: "t1", name: "Trabajo" },
        { id: "t3", name: "sma-1" },
      ],
    });
  });

  it("needs a space before the hash", () => {
    expect(parse("Canal C#trabajo")).toBeNull();
  });

  it.each([
    ["Informe !1", 1],
    ["Informe !3", 3],
  ])("%s → %i", (text, priority) => {
    expectCapture(text, { title: "Informe", priority });
  });

  it("leaves other exclamations alone", () => {
    expect(parse("Informe !4")).toBeNull();
    expect(parse("¡Hola!")).toBeNull();
  });

  it("without a tag lookup leaves every hash in the title", () => {
    expect(parseNatural("Informe #trabajo", { today: TODAY })).toBeNull();
  });
});

describe("parseNatural: repetition", () => {
  it.each([
    ["Regar cada día", "FREQ=DAILY;INTERVAL=1"],
    ["Regar todos los días", "FREQ=DAILY;INTERVAL=1"],
    ["Regar diariamente", "FREQ=DAILY;INTERVAL=1"],
    ["Regar cada 2 semanas", "FREQ=WEEKLY;INTERVAL=2"],
    ["Regar cada tres días", "FREQ=DAILY;INTERVAL=3"],
    ["Regar todos los meses", "FREQ=MONTHLY;INTERVAL=1"],
    ["Regar cada año", "FREQ=YEARLY;INTERVAL=1"],
    ["Regar semanalmente", "FREQ=WEEKLY;INTERVAL=1"],
  ])("%s → %s", (text, rrule) => {
    expectCapture(text, { title: "Regar", rrule, date: TODAY });
  });

  it("weekdays start the task on the first matching day", () => {
    expectCapture("Gimnasio los lunes y jueves", {
      title: "Gimnasio",
      rrule: "FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,TH",
      days: ["MO", "TH"],
      date: "2026-11-23",
    });
    expectCapture("Gimnasio cada viernes", {
      rrule: "FREQ=WEEKLY;INTERVAL=1;BYDAY=FR",
      date: TODAY,
    });
    expectCapture("Paseo los sábados, domingos", {
      days: ["SA", "SU"],
    });
  });

  it.each<[string, DayCode[]]>([
    ["Fichar entre semana", ["MO", "TU", "WE", "TH", "FR"]],
    ["Fichar de lunes a viernes", ["MO", "TU", "WE", "TH", "FR"]],
    ["Fichar los fines de semana", ["SA", "SU"]],
  ])("%s", (text, days) => {
    expectCapture(text, { title: "Fichar", days });
  });

  it("a daily habit falls on every day", () => {
    expect(parse("Meditar cada día")?.days).toHaveLength(7);
    expect(parse("Meditar cada 2 días")?.days).toEqual([]);
  });

  it("rejects intervals the backend refuses", () => {
    expect(parse("Regar cada 400 días")).toBeNull();
  });
});

describe("parseNatural: everything together", () => {
  it("reads a full sentence and cleans the title", () => {
    expect(parse("Dentista, mañana a las 5 durante 1h #salud !2")).toEqual({
      title: "Dentista",
      date: "2026-11-21",
      time: "17:00",
      duration: 60,
      tags: [{ id: "t2", name: "salud" }],
      priority: 2,
      rrule: null,
      days: [],
    });
  });

  it("takes one detail of each kind, by rule order", () => {
    expectCapture("Llamar hoy o mañana", {
      title: "Llamar hoy o",
      date: "2026-11-21",
    });
  });
});

describe("date helpers", () => {
  it("addDays crosses months and years", () => {
    expect(addDays("2026-12-31", 1)).toBe("2027-01-01");
    expect(addDays("2027-03-01", -1)).toBe("2027-02-28");
  });

  it("firstOn finds the next matching weekday", () => {
    expect(firstOn(["MO"], TODAY)).toBe("2026-11-23");
    expect(firstOn(["FR"], TODAY)).toBe(TODAY);
  });
});

describe("taskFieldsOf", () => {
  it("maps a capture to the fields of a new task", () => {
    expect(
      taskFieldsOf(parse("Dentista mañana a las 5 durante 1h #salud !2")!),
    ).toEqual({
      dueDate: "2026-11-21",
      dueTime: "17:00",
      durationMinutes: 60,
      tagIds: ["t2"],
      priority: 2,
    });
    expect(taskFieldsOf(parse("Informe #trabajo")!)).toEqual({
      tagIds: ["t1"],
    });
    expect(taskFieldsOf(parse("Regar cada día")!)).toEqual({
      dueDate: TODAY,
      rrule: "FREQ=DAILY;INTERVAL=1",
    });
  });
});
