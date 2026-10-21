//! A machine's or a project's row (its heading), which stands for its coordinator, drawn as one line of the task list:
//! the coordinator's own mark in the marks' column, `◈`, in the colour of where it stands as a task's mark is (amber
//! working, red needs you, plain idle, dim not started or stopped; autopilot's `◎` while that is on), no rule to the
//! edge, and on the right how long ago it last said something (autopilot's timing while on). Nothing under it.
use super::pilot::Shown;
use crate::board::state::State;
use crate::board::theme::Theme;
use crate::board::update::coords::{look_of, of_head, Target};
use crate::shown::ink::{Ink, Inked};

/// The coordinators' mark: a diamond in a diamond, of the family of a task's marks (round, a diamond or a sign) and
/// one cell wide as they are; not a round dot, which herdr's agents have.
pub const COORD_MARK: &str = "◈";
/// Its mark, in the colour of where its coordinator stands.
fn mark(s: &State, c: &Target) -> Inked {
    if c.host.is_empty() && !s.data.can_dispatch { return Inked::of(COORD_MARK, Ink::Muted); }
    let v = look_of(s, c);
    let (state, err) = v.as_ref().map(|v| (v.state.clone(), !v.error.is_empty())).unwrap_or(("none".into(), false));
    let ink = if err { Ink::Red } else { match state.as_str() { "working" | "starting" => Ink::Yellow, "blocked" => Ink::Red, "idle" => Ink::Text, _ => Ink::Muted } };
    let m = Inked::of(COORD_MARK, ink);
    if ink == Ink::Red { m.bold() } else { m }
}
/// When it last said something (0: not known).
fn said_at(s: &State, c: &Target) -> i64 { look_of(s, c).and_then(|v| v.talk.last().map(|x| x.at)).unwrap_or(0) }
/// What the row of the coordinator a machine's or a project's heading stands for shows: autopilot's while it is on
/// (its mark and timing), else its own mark and time. None for a heading with no coordinator. Never a line under it.
pub fn shown(s: &State, t: &Theme, id: &str) -> Option<Shown> {
    if let Some(p) = super::pilot::shown(s, t, id) { return Some(Shown { under: Inked::new(), ..p }); }
    let c = of_head(s, id)?;
    let at = said_at(s, &c);
    let cols = if at > 0 { vec![("time", Inked::of(format!("{} ago", crate::shown::words::age(s.now - at)), Ink::Muted))] } else { vec![] };
    Some(Shown { mark: mark(s, &c), cols, under: Inked::new() })
}
