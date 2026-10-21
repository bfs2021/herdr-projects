//! A machine's or a project's row (its heading), which stands for its coordinator, drawn as a task agent's row: the
//! mark of where its coordinator stands in the marks' column (the chat's marks for a coordinator or an agent: working,
//! idle, needs you, stopped, not started; autopilot's `◎` while that is on), no rule to the edge, on the right how long
//! ago it last said something (autopilot's timing while on), and under its name the last thing it said. One that never
//! said anything has no second line.
use super::pilot::Shown;
use crate::board::state::State;
use crate::board::theme::Theme;
use crate::board::update::coords::{look_of, of_head, Target};
use crate::shown::ink::{Ink, Inked};

/// Its coordinator's mark, as the chat's line marks it.
fn mark(s: &State, t: &Theme, c: &Target) -> Inked {
    if c.host.is_empty() && !s.data.can_dispatch { return Inked::of(t.mark("unknown", ""), Ink::Muted); }
    let v = look_of(s, c);
    let (state, err) = v.as_ref().map(|v| (v.state.clone(), !v.error.is_empty())).unwrap_or(("none".into(), false));
    let (kind, ink) = if err { ("error", Ink::Red) } else { match state.as_str() { "working" | "starting" => ("working", Ink::Yellow), "blocked" => ("blocked", Ink::Red), "idle" => ("idle", Ink::Green), _ => ("unknown", Ink::Muted) } };
    let m = Inked::of(t.mark(kind, ""), ink);
    if ink == Ink::Red { m.bold() } else { m }
}
/// The last thing it said, on one line, and when (0: not known).
fn said(s: &State, c: &Target) -> (String, i64) {
    let Some(v) = look_of(s, c) else { return (String::new(), 0) };
    let last = v.talk.last().cloned();
    let text = if !v.said.trim().is_empty() { v.said.clone() } else { last.as_ref().map(|x| x.text.clone()).unwrap_or_default() };
    (text.lines().map(str::trim).find(|l| !l.is_empty()).map(crate::shown::words::clean).unwrap_or_default(), last.map(|x| x.at).unwrap_or(0))
}
/// What the row of the coordinator a machine's or a project's heading stands for shows: autopilot's while it is on,
/// else its own (its mark, its time, its last line). None for a heading with no coordinator.
pub fn shown(s: &State, t: &Theme, id: &str) -> Option<Shown> {
    if let Some(p) = super::pilot::shown(s, t, id) { return Some(p); }
    let c = of_head(s, id)?;
    let (line, at) = said(s, &c);
    let cols = if at > 0 { vec![("time", Inked::of(format!("{} ago", crate::shown::words::age(s.now - at)), Ink::Muted))] } else { vec![] };
    Some(Shown { mark: mark(s, t, &c), cols, under: Inked::of(line, Ink::Soft) })
}
/// Does its row take a second line: autopilot's, or the last thing its coordinator said?
pub fn has_under(s: &State, id: &str) -> bool {
    let Some(c) = of_head(s, id) else { return false };
    crate::board::update::pilot::on(s, &c) || !said(s, &c).0.is_empty()
}
