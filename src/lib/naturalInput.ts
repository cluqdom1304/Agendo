// Quick capture in plain Spanish (D-96): «Dentista mañana a las 5 durante 1h #salud !2».
// Pure: the day it reads relative dates from comes in as an argument (the caller's clock).
// Each kind of detail is taken once (the first match) and cut out of the title; what is left
// is the title. Nothing is taken when the title would end up empty, so «mañana» alone is a
// task called «mañana».

import { buildRule, DAY_CODES, type DayCode, type Freq } from "./recurrence";

export interface CapturedTag {
  id: string;
  name: string;
}

export interface Capture {
  title: string;
  /** `YYYY-MM-DD`: the typed day, else the fallback day when a time or repetition needs one. */
  date: string | null;
  /** `HH:MM`. */
  time: string | null;
  /** Minutes; only with a time, and ending the same day. */
  duration: number | null;
  tags: CapturedTag[];
  /** 0 none … 3 urgent (D-93): `!1` media, `!2` alta, `!3` urgente. */
  priority: number;
  rrule: string | null;
  /** Weekdays the repetition falls on (habits): all for «cada día», the chosen ones if weekly. */
  days: DayCode[];
}

export interface CaptureOptions {
  /** Today, `YYYY-MM-DD`, from the caller's clock. */
  today: string;
  /** Day used when only a time or a repetition is typed (the calendar slot, the list day). */
  fallbackDate?: string;
  /** Id of the existing tag with that name; unknown `#words` stay in the title. */
  tagId?: (name: string) => string | undefined;
}

const WORD = "[\\p{L}\\p{N}]";
const B = `(?<!${WORD})`;
const E = `(?!${WORD})`;
const re = (body: string) => new RegExp(`${B}(?:${body})${E}`, "giu");
const exact = (body: string) => new RegExp(`^(?:${body})$`, "i");

const NUMBERS: Record<string, number> = {
  un: 1,
  una: 1,
  uno: 1,
  dos: 2,
  tres: 3,
  cuatro: 4,
  cinco: 5,
  seis: 6,
  siete: 7,
  ocho: 8,
  nueve: 9,
  diez: 10,
};
const NUM = `\\d{1,3}|${Object.keys(NUMBERS).join("|")}`;
const toNumber = (text: string) => NUMBERS[text.toLowerCase()] ?? Number(text);

/** Monday first, like `DAY_CODES`; plurals for «los sábados». */
const DAY_WORDS = [
  "lunes",
  "martes",
  "mi[eé]rcoles",
  "jueves",
  "viernes",
  "s[aá]bados?",
  "domingos?",
];
const DAY = DAY_WORDS.join("|");
const DAY_RES = DAY_WORDS.map(exact);
const dayIndex = (text: string) => DAY_RES.findIndex((day) => day.test(text));

const MONTH_WORDS = [
  "ene(?:ro)?",
  "feb(?:rero)?",
  "mar(?:zo)?",
  "abr(?:il)?",
  "may(?:o)?",
  "jun(?:io)?",
  "jul(?:io)?",
  "ago(?:sto)?",
  "sep(?:t(?:iembre)?)?|set(?:iembre)?",
  "oct(?:ubre)?",
  "nov(?:iembre)?",
  "dic(?:iembre)?",
];
const MONTH_RES = MONTH_WORDS.map(exact);

const pad = (value: number) => String(value).padStart(2, "0");

function parts(iso: string): [number, number, number] {
  const [year, month, day] = iso.split("-").map(Number);
  return [year, month, day];
}

/** `YYYY-MM-DD` of a valid calendar day, else `null`. */
function isoOf(year: number, month: number, day: number): string | null {
  const date = new Date(Date.UTC(year, month - 1, day));
  return date.getUTCMonth() === month - 1 && date.getUTCDate() === day
    ? `${String(year).padStart(4, "0")}-${pad(month)}-${pad(day)}`
    : null;
}

export function addDays(iso: string, count: number): string {
  const [year, month, day] = parts(iso);
  const date = new Date(Date.UTC(year, month - 1, day + count));
  return `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())}`;
}

/** Same day `count` months later, or the month's last day when it is shorter. */
function addMonths(iso: string, count: number): string {
  const [year, month, day] = parts(iso);
  const index = year * 12 + month - 1 + count;
  const last = new Date(Date.UTC(Math.floor(index / 12), (index % 12) + 1, 0));
  return addDays(
    `${last.getUTCFullYear()}-${pad(last.getUTCMonth() + 1)}-01`,
    Math.min(day, last.getUTCDate()) - 1,
  );
}

/** 0 = Monday … 6 = Sunday. */
function weekday(iso: string): number {
  const [year, month, day] = parts(iso);
  return (new Date(Date.UTC(year, month - 1, day)).getUTCDay() + 6) % 7;
}

/** The first day on or after `from` that falls on one of `days`. */
export function firstOn(days: readonly DayCode[], from: string): string {
  for (let step = 0; step < 7; step++) {
    const date = addDays(from, step);
    if (days.includes(DAY_CODES[weekday(date)])) return date;
  }
  return from;
}

/** A day and month without year: this year's, or next year's once it has passed. */
function nextDayMonth(
  today: string,
  month: number,
  day: number,
): string | null {
  const year = parts(today)[0];
  const date = isoOf(year, month, day);
  if (date === null && !(month === 2 && day === 29)) return null;
  if (date !== null && date >= today) return date;
  for (let next = year + 1; next <= year + 8; next++) {
    const later = isoOf(next, month, day);
    if (later) return later;
  }
  return null;
}

function fullYear(text: string | undefined): number | null {
  if (!text) return null;
  return text.length === 2 ? 2000 + Number(text) : Number(text);
}

interface Found {
  date: string | null;
  time: string | null;
  duration: number | null;
  priority: number;
  freq: Freq | null;
  interval: number;
  days: DayCode[];
}

type Taker = (match: RegExpExecArray, found: Found, today: string) => boolean;

const WEEKDAYS: DayCode[] = ["MO", "TU", "WE", "TH", "FR"];
const UNIT_FREQ: [RegExp, Freq][] = [
  [/^d/i, "DAILY"],
  [/^s/i, "WEEKLY"],
  [/^m/i, "MONTHLY"],
  [/^a/i, "YEARLY"],
];
const freqOf = (unit: string) =>
  UNIT_FREQ.find(([test]) => test.test(unit))?.[1] ?? "DAILY";

function setRepeat(
  found: Found,
  freq: Freq,
  interval = 1,
  days: DayCode[] = [],
) {
  found.freq = freq;
  found.interval = interval;
  found.days = days;
  return interval >= 1 && interval <= 365;
}

/** Hours of «a las 5»: 1 to 7 without «de la mañana» or «am» are read as the afternoon. */
function setTime(
  found: Found,
  hourText: string,
  minuteText: string | undefined,
  extra: string | undefined,
  meridiem: string | undefined,
  guess: boolean,
): boolean {
  let hour = Number(hourText);
  let minute = minuteText ? Number(minuteText) : 0;
  if (extra) minute += /media/i.test(extra) ? 30 : 15;
  const part = meridiem?.toLowerCase().replace(/[\s.]/g, "") ?? "";
  if (part) {
    if (hour < 1 || hour > 12) return false;
    const evening = /^(pm|tarde|noche)$/.test(part);
    if (evening && hour < 12) hour += 12;
    if ((!evening || part === "noche") && hour === 12) hour = 0;
  } else if (guess && hour >= 1 && hour <= 7) {
    hour += 12;
  }
  if (hour > 23 || minute > 59) return false;
  found.time = `${pad(hour)}:${pad(minute)}`;
  return true;
}

function setDuration(found: Found, minutes: number): boolean {
  if (minutes < 1 || minutes > 1440) return false;
  found.duration = minutes;
  return true;
}

const MERIDIEM =
  "\\s*(am|pm|a\\.\\s?m\\.|p\\.\\s?m\\.)|\\s+de\\s+la\\s+(mañana|tarde|noche|madrugada)";
const FOR = "(?:(?:durante|por)\\s+)?";

/** Tried in order; each kind is taken once. Time runs before the date so «de la mañana»
 * is never read as tomorrow, and the duration after the time so «a las 17h» stays a time. */
const RULES: [keyof Found, RegExp, Taker][] = [
  [
    "priority",
    re("(?<=^|\\s)!([1-3])"),
    (m, f) => ((f.priority = Number(m[1])), true),
  ],
  [
    "freq",
    re(
      `(?:cada|todos\\s+los|todas\\s+las)\\s+(?:(${NUM})\\s+)?(d[ií]as?|semanas?|mes(?:es)?|a[nñ]os?)`,
    ),
    (m, f) => setRepeat(f, freqOf(m[2]), m[1] ? toNumber(m[1]) : 1),
  ],
  ["freq", re("diariamente|a\\s+diario"), (_m, f) => setRepeat(f, "DAILY")],
  ["freq", re("semanalmente"), (_m, f) => setRepeat(f, "WEEKLY")],
  ["freq", re("mensualmente"), (_m, f) => setRepeat(f, "MONTHLY")],
  ["freq", re("anualmente"), (_m, f) => setRepeat(f, "YEARLY")],
  [
    "freq",
    re(
      "de\\s+lunes\\s+a\\s+viernes|entre\\s+semana|(?:los\\s+)?d[ií]as\\s+laborables",
    ),
    (_m, f) => setRepeat(f, "WEEKLY", 1, WEEKDAYS),
  ],
  [
    "freq",
    re("(?:los|cada|todos\\s+los)\\s+fines\\s+de\\s+semana"),
    (_m, f) => setRepeat(f, "WEEKLY", 1, ["SA", "SU"]),
  ],
  [
    "freq",
    re(
      `(?:cada|todos\\s+los|todas\\s+las|los)\\s+(?:${DAY})(?:\\s*(?:,|y)\\s*(?:el\\s+|los\\s+)?(?:${DAY}))*`,
    ),
    (m, f) => {
      const chosen = m[0]
        .split(/[\s,]+/)
        .map(dayIndex)
        .filter((index) => index >= 0);
      return setRepeat(
        f,
        "WEEKLY",
        1,
        DAY_CODES.filter((_c, i) => chosen.includes(i)),
      );
    },
  ],
  [
    "time",
    re(
      `(?:a|sobre|hacia|desde)\\s+las?\\s+(\\d{1,2})(?:[:.](\\d{2})|\\s*hs?${E})?(?:\\s+y\\s+(media|cuarto))?(?:${MERIDIEM})?`,
    ),
    (m, f) => setTime(f, m[1], m[2], m[3], m[4] ?? m[5], true),
  ],
  [
    "time",
    re("(?:a\\s+|al\\s+)?mediod[ií]a"),
    (_m, f) => setTime(f, "12", "00", undefined, undefined, false),
  ],
  [
    "time",
    re(`(\\d{1,2}):(\\d{2})(?:${MERIDIEM})?`),
    (m, f) => setTime(f, m[1], m[2], undefined, m[3] ?? m[4], false),
  ],
  [
    "time",
    re(`(\\d{1,2})(?:${MERIDIEM})`),
    (m, f) => setTime(f, m[1], undefined, undefined, m[2] ?? m[3], false),
  ],
  [
    "duration",
    re(`${FOR}(?:una\\s+)?hora\\s+y\\s+media`),
    (_m, f) => setDuration(f, 90),
  ],
  ["duration", re(`${FOR}media\\s+hora`), (_m, f) => setDuration(f, 30)],
  ["duration", re(`${FOR}una\\s+hora`), (_m, f) => setDuration(f, 60)],
  [
    "duration",
    re(
      `${FOR}(\\d{1,2})\\s*h(?:oras?)?(?:(\\d{2})|\\s*(?:y\\s+)?(\\d{1,2})\\s*min(?:utos?|s)?)?`,
    ),
    (m, f) => setDuration(f, Number(m[1]) * 60 + Number(m[2] ?? m[3] ?? 0)),
  ],
  [
    "duration",
    re(`${FOR}(\\d{1,4})\\s*min(?:utos?|s)?`),
    (m, f) => setDuration(f, Number(m[1])),
  ],
  [
    "date",
    re("pasado\\s+mañana"),
    (_m, f, today) => ((f.date = addDays(today, 2)), true),
  ],
  [
    "date",
    new RegExp(`(?<!(?:la|esta)\\s+)${B}mañana${E}`, "giu"),
    (_m, f, today) => ((f.date = addDays(today, 1)), true),
  ],
  ["date", re("hoy"), (_m, f, today) => ((f.date = today), true)],
  [
    "date",
    re(`(?:en|dentro\\s+de)\\s+(${NUM})\\s+(d[ií]as?|semanas?|mes(?:es)?)`),
    (m, f, today) => {
      const count = toNumber(m[1]);
      const freq = freqOf(m[2]);
      f.date =
        freq === "MONTHLY"
          ? addMonths(today, count)
          : addDays(today, count * (freq === "WEEKLY" ? 7 : 1));
      return true;
    },
  ],
  [
    "date",
    re("(?:la\\s+)?(?:semana\\s+que\\s+viene|pr[oó]xima\\s+semana)"),
    (_m, f, today) => ((f.date = addDays(today, 7 - weekday(today))), true),
  ],
  [
    "date",
    re("(?:el\\s+)?(\\d{1,2})\\/(\\d{1,2})(?:\\/(\\d{4}|\\d{2}))?"),
    (m, f, today) => {
      const year = fullYear(m[3]);
      f.date =
        year === null
          ? nextDayMonth(today, Number(m[2]), Number(m[1]))
          : isoOf(year, Number(m[2]), Number(m[1]));
      return f.date !== null;
    },
  ],
  [
    "date",
    re(
      "(?:el\\s+)?(?:d[ií]a\\s+)?(\\d{1,2})\\s+(?:de\\s+)?(\\p{L}+)\\.?(?:\\s+(?:de\\s+)?(\\d{4}))?",
    ),
    (m, f, today) => {
      const month = MONTH_RES.findIndex((test) => test.test(m[2])) + 1;
      if (month === 0) return false;
      f.date = m[3]
        ? isoOf(Number(m[3]), month, Number(m[1]))
        : nextDayMonth(today, month, Number(m[1]));
      return f.date !== null;
    },
  ],
  [
    "date",
    re(
      `(?:(este|el(?:\\s+pr[oó]ximo)?|pr[oó]ximo)\\s+)?(${DAY})(?:\\s+que\\s+viene|\\s+pr[oó]ximo)?`,
    ),
    (m, f, today) => {
      const ahead = (dayIndex(m[2]) - weekday(today) + 7) % 7;
      f.date = addDays(
        today,
        ahead === 0 && !/^este$/i.test(m[1] ?? "") ? 7 : ahead,
      );
      return true;
    },
  ],
  [
    "date",
    re("el\\s+(?:d[ií]a\\s+)?(\\d{1,2})(?![/:.]\\d|\\s+de\\s)"),
    (m, f, today) => {
      const day = Number(m[1]);
      const [year, month] = parts(today);
      for (let step = 0; step < 12; step++) {
        const index = year * 12 + month - 1 + step;
        const date = isoOf(Math.floor(index / 12), (index % 12) + 1, day);
        if (date && date >= today) {
          f.date = date;
          return true;
        }
      }
      return false;
    },
  ],
];

const TAG = /(?<=^|\s)#([\p{L}\p{N}_-]+)/gu;

/** What `text` asks for, or `null` when nothing was recognised (create it as typed). */
export function parseNatural(
  text: string,
  options: CaptureOptions,
): Capture | null {
  const { today, tagId } = options;
  const tags: CapturedTag[] = [];
  let rest = tagId
    ? text.replace(TAG, (whole, name: string) => {
        const id = tagId(name);
        if (id === undefined) return whole;
        if (!tags.some((tag) => tag.id === id)) tags.push({ id, name });
        return " ";
      })
    : text;
  const found: Found = {
    date: null,
    time: null,
    duration: null,
    priority: 0,
    freq: null,
    interval: 1,
    days: [],
  };
  const taken = new Set<keyof Found>();
  for (const [kind, pattern, take] of RULES) {
    if (taken.has(kind) || (kind === "duration" && found.time === null))
      continue;
    pattern.lastIndex = 0;
    const match = [...rest.matchAll(pattern)].find((candidate) =>
      take(candidate, found, today),
    );
    if (!match) continue;
    taken.add(kind);
    rest = `${rest.slice(0, match.index)} ${rest.slice(match.index + match[0].length)}`;
  }
  const title = rest
    .replace(/\s+/g, " ")
    .replace(/^[\s,;:–-]+|[\s,;:–-]+$/g, "");
  if (title === "" || (taken.size === 0 && tags.length === 0)) return null;

  const fallback = options.fallbackDate ?? today;
  let date = found.date;
  if (date === null && found.freq === "WEEKLY" && found.days.length > 0) {
    date = firstOn(found.days, fallback);
  } else if (date === null && (found.time !== null || found.freq !== null)) {
    date = fallback;
  }
  const rrule = found.freq
    ? buildRule({
        freq: found.freq,
        interval: found.interval,
        until: "",
        days: found.days,
        count: null,
      })
    : null;
  const days =
    found.freq === "DAILY" && found.interval === 1
      ? [...DAY_CODES]
      : found.days;
  const minutesLeft = found.time
    ? 1440 - Number(found.time.slice(0, 2)) * 60 - Number(found.time.slice(3))
    : 0;
  return {
    title,
    date,
    time: found.time,
    duration:
      found.duration === null ? null : Math.min(found.duration, minutesLeft),
    tags,
    priority: found.priority,
    rrule,
    days,
  };
}
