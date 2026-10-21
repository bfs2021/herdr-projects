//! A coordinator as a row of the list of coordinators (the kinds row's `coordinators`): its dot, its name (another
//! machine's with that machine beside it), where it stands on the right, and under it the last thing it said.
use crate::board::state::State;
use crate::board::update::coords::{look_of, of_head};
use crate::shown::ink::{Ink, Inked};

/// Its two lines, `width` cells wide.
pub fn coord_lines(s: &State, id: &str, width: usize) -> Vec<Inked> {
    let Some(t) = of_head(s, id) else { return vec![] };
    // This machine's Coordinator is read here; another machine's main one is what it sent (update/coords.rs: look_of).
    let main = id == crate::board::update::pilot::main_id("");
    let (mark, ink, word) = super::scope_bar::dot(s, (!main).then_some(&t));
    let mut name = Inked::plain(" ").add(Inked::of(mark, ink)).add(Inked::plain(" ")).add(Inked::of(t.name.clone(), Ink::Text));
    if !t.host.is_empty() && !t.name.contains(&t.host) { name = name.add(Inked::of(format!(" · {}", crate::board::update::remotes::shown(s, &t.host)), Ink::Muted)); }
    let room = width.saturating_sub(crate::text::width(word) + 2);
    let first = name.cut(room).fit(room).add(Inked::of(format!(" {word}"), ink));
    let v = if main { Some(s.data.coord.clone()) } else { look_of(s, &t) };
    let said = v.as_ref().map(|v| if !v.said.trim().is_empty() { v.said.clone() } else { v.talk.last().map(|x| x.text.clone()).unwrap_or_default() }).unwrap_or_default();
    let line = said.lines().map(str::trim).find(|l| !l.is_empty()).map(crate::shown::words::clean)
        .unwrap_or_else(|| if word == "not started" { "not started: your first message starts it".into() } else { "nothing said yet".into() });
    vec![first, Inked::plain("   ").add(Inked::of(crate::text::cut(&line, width.saturating_sub(4).max(1)), Ink::Soft))]
}
