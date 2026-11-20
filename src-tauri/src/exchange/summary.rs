//! Export of the weekly summary (F26-03, D-100) as one Markdown file.

use chrono::{Datelike, Local, TimeZone};

use super::export::write_whole;
use super::*;
use crate::{repo::model::WeekSummary, services::summary};

const DAY_NAMES: [&str; 7] = ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"];

fn spoken(date: &str) -> String {
    match dates::parse_date(date) {
        Ok(day) => day.format("%d/%m/%Y").to_string(),
        Err(_) => date.to_string(),
    }
}

/// «23 s», «5 min 20 s» or «1 h 25 min», as the task rows show time.
pub(super) fn duration(seconds: i64) -> String {
    let total = seconds.max(0);
    if total < 60 {
        return format!("{total} s");
    }
    if total < 3600 {
        return match total % 60 {
            0 => format!("{} min", total / 60),
            rest => format!("{} min {rest} s", total / 60),
        };
    }
    let minutes = (total + 30) / 60;
    match (minutes / 60, minutes % 60) {
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes} min"),
    }
}

/// The summary as Markdown. `tag_names` turns tag ids into names; a tag that no longer exists
/// shows as «(etiqueta eliminada)».
pub fn markdown(summary: &WeekSummary, tag_names: &HashMap<String, String>) -> String {
    let mut out = format!(
        "# Resumen de la semana del {} al {}\n\n",
        spoken(&summary.start),
        spoken(&summary.end)
    );
    out.push_str("## Tareas\n\n");
    out.push_str(&format!(
        "- Completadas: {}\n- Pendientes: {}\n- Vencidas: {}\n",
        summary.completed, summary.pending, summary.overdue
    ));
    if !summary.by_tag.is_empty() {
        out.push_str("\n### Por etiqueta\n\n");
        out.push_str("| Etiqueta | Completadas | Pendientes | Vencidas |\n|---|---:|---:|---:|\n");
        let mut rows: Vec<(String, &crate::repo::model::TagTally)> = summary
            .by_tag
            .iter()
            .map(|tally| {
                let name = match &tally.tag_id {
                    None => "Sin etiqueta".to_string(),
                    Some(id) => tag_names
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| "(etiqueta eliminada)".into()),
                };
                (name.replace('|', "\\|"), tally)
            })
            .collect();
        rows.sort_by_key(|row| row.0.to_lowercase());
        for (name, tally) in rows {
            out.push_str(&format!(
                "| {name} | {} | {} | {} |\n",
                tally.completed, tally.pending, tally.overdue
            ));
        }
    }
    out.push_str("\n## Eventos\n\n");
    out.push_str(&format!("- Eventos: {}\n", summary.events));
    if summary.event_minutes > 0 {
        out.push_str(&format!(
            "- Tiempo en eventos con hora: {}\n",
            duration(summary.event_minutes * 60)
        ));
    }
    out.push_str("\n## Hábitos\n\n");
    if summary.habits.is_empty() {
        out.push_str("Sin hábitos con días previstos esta semana.\n");
    }
    for habit in &summary.habits {
        let percent = habit.done * 100 / habit.scheduled.max(1);
        out.push_str(&format!(
            "- {}: {} de {} ({percent} %)\n",
            habit.title.replace('\n', " "),
            habit.done,
            habit.scheduled
        ));
    }
    out.push_str("\n## Enfoque\n\n");
    if summary.focus_seconds == 0 {
        out.push_str("Sin tiempo de enfoque registrado.\n");
    } else {
        out.push_str(&format!(
            "- Total: {} en {} sesiones\n",
            duration(summary.focus_seconds),
            summary.focus_sessions
        ));
        if let Ok(start) = dates::parse_date(&summary.start) {
            for (offset, seconds) in summary.focus_by_day.iter().enumerate() {
                if *seconds > 0 {
                    let day = start + chrono::Duration::days(offset as i64);
                    let name = DAY_NAMES[day.weekday().num_days_from_monday() as usize];
                    out.push_str(&format!(
                        "  - {name} {}: {}\n",
                        day.format("%d/%m"),
                        duration(*seconds)
                    ));
                }
            }
        }
    }
    out
}

/// Writes the summary of the week of `date` to `target`. The file is written beside itself first
/// so an older one is replaced only by a whole one.
pub fn export_summary(state: &AppState, date: &str, first_day: u8, target: &Path) -> AppResult<()> {
    export_summary_in(state, date, first_day, target, &Local)
}

pub fn export_summary_in<Tz: TimeZone>(
    state: &AppState,
    date: &str,
    first_day: u8,
    target: &Path,
    zone: &Tz,
) -> AppResult<()> {
    let week = summary::week_in(state, date, first_day, zone)?;
    let names: HashMap<String, String> = state.db.read(|conn| {
        Ok(tags::list(conn)?
            .into_iter()
            .map(|entry| (entry.tag.id, entry.tag.name))
            .collect())
    })?;
    write_whole(target, &markdown(&week, &names))
}
