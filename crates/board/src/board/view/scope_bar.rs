//! The scope bar on top of the board, `● All › ● vm-str › ● herdr-projects`, and the kinds row under it, `all · tasks ·
//! agents · coordinators · automations`, with how many tasks stand in each group on its right. Each step has its
//! coordinator's dot; after the last step, what the next one could be (greyed), or the picker while it is open. Where
//! each step, choice and kind is drawn is kept for a click (update/scope.rs: click).
use crate::board::state::State;
use crate::board::theme::Theme;
use crate::board::update::coords::{look_of, Target};
use crate::board::update::{kinds, pilot, remotes, scope};
use crate::shown::ink::{Ink, Inked};
use crate::shown::words::repo_label;

/// A coordinator's dot and its word: working, idle, needs you, on autopilot or not started. None: the Coordinator.
pub fn dot(s: &State, t: Option<&Target>) -> (&'static str, Ink, &'static str) {
    let main = Target::default();
    let t = t.unwrap_or(&main);
    if t.host.is_empty() && !s.data.can_dispatch { return ("○", Ink::Muted, "off here"); }
    if pilot::on(s, t) { return (super::pilot::PILOT_MARK, Ink::Accent, "on autopilot"); }
    let v = look_of(s, t);
    let (state, err) = v.as_ref().map(|v| (v.state.as_str(), !v.error.is_empty())).unwrap_or(("none", false));
    if err || state == "blocked" { return ("●", Ink::Red, "needs you"); }
    match state { "working" | "starting" => ("●", Ink::Yellow, "working"), "idle" => ("●", Ink::Green, "idle"), "exited" => ("○", Ink::Muted, "stopped"), _ => ("○", Ink::Muted, "not started") }
}
/// One line being laid out, with where each clickable piece of it lands.
struct Line { text: Inked, hits: Vec<(usize, usize, usize, String)>, row: usize }
impl Line {
    fn put(&mut self, piece: Inked, what: &str) {
        let from = self.text.width();
        self.text = std::mem::take(&mut self.text).add(piece);
        if !what.is_empty() { self.hits.push((self.row, from, self.text.width(), what.to_string())); }
    }
}
/// The steps: each its dot and its name, the one you are on in bold.
fn steps(s: &State, l: &mut Line) {
    let mut all: Vec<(Option<Target>, String)> = vec![(None, "All".into())];
    if s.level >= 1 { all.push((crate::board::update::coords::of_head(s, &crate::board::update::heads::machine_id(&s.scope)), remotes::shown(s, &s.scope))); }
    if s.level >= 2 { all.push((crate::board::update::coords::of_head(s, &scope::repo_id(&s.scope, &s.project)), repo_label(&s.project))); }
    let last = all.len() - 1;
    for (n, (t, name)) in all.into_iter().enumerate() {
        if n > 0 { l.put(Inked::of(" › ", Ink::Muted), ""); }
        let (mark, ink, _) = dot(s, t.as_ref());
        let named = if n == last { Inked::of(name, Ink::Text).bold() } else { Inked::of(name, Ink::Soft) };
        l.put(Inked::of(mark, ink).add(Inked::plain(" ")).add(named), &format!("level\t{n}"));
    }
}
/// After the last step: the picker while it is open (what is typed, then the matches, the picked one lit), else what the
/// next step could be, greyed.
fn next(s: &State, th: &Theme, l: &mut Line, w: usize) {
    if s.level >= 2 { return; }
    let what = if s.level == 0 { "machine" } else { "project" };
    let open = s.pick.as_ref();
    let list = if open.is_some() { scope::matches(s) } else { scope::candidates(s) };
    l.put(Inked::of(" ›", Ink::Muted), "more");
    if let Some(p) = open { l.put(Inked::plain(" ").add(Inked::of(format!("{}▏", p.typed), Ink::Accent)), ""); }
    if list.is_empty() { if open.is_some() { l.put(Inked::of("  nothing matches", Ink::Muted), ""); } return; }
    for (n, (id, label)) in list.iter().enumerate() {
        let far = s.level == 0 && s.remotes.get(id).is_some_and(|m| !m.ok);
        let label = if far { format!("{label} (no answer)") } else { label.clone() };
        let gap = if n == 0 && open.is_none() { " " } else { "  " };
        if l.text.width() + gap.len() + crate::text::width(&label) + 4 > w && n > 0 { l.put(Inked::of(format!("  +{}", list.len() - n), Ink::Muted), "more"); break; }
        l.put(Inked::plain(gap), "");
        let lit = open.is_some_and(|p| p.at.min(list.len() - 1) == n);
        let named = if lit { let x = Inked::of(label, Ink::Accent).bold(); if th.paints_selection() { x.on(Ink::Selection) } else { x.inv() } } else { Inked::of(label, Ink::Muted) };
        l.put(named, &format!("{what}\t{id}"));
    }
}
/// The kinds row: each kind, the one shown in the accent colour (lit while the row has the keys).
fn kinds_row(s: &State, th: &Theme, l: &mut Line, w: usize) {
    let short = w < 56;
    l.put(Inked::plain(" "), "");
    for (n, k) in kinds::KINDS.iter().enumerate() {
        if n > 0 { l.put(Inked::of(" · ", Ink::Muted), ""); }
        let name = if short { match *k { "coordinators" => "coords", "automations" => "autos", k => k } } else { k };
        let on = n == s.kind;
        let named = if on { Inked::of(name, Ink::Accent).bold() } else { Inked::of(name, Ink::Muted) };
        let named = if on && s.kinds_focus { if th.paints_selection() { named.on(Ink::Selection) } else { named.inv() } } else { named };
        l.put(named, &format!("kind\t{n}"));
    }
}
/// The bar, the kinds row and, on its right where it fits (else on a line of its own), `tally`: how many tasks stand
/// in each group. With where a click on each piece goes, by line.
pub fn bar_lines(s: &State, th: &Theme, w: usize, tally: &str) -> (Vec<Inked>, Vec<(usize, usize, usize, String)>) {
    let mut bar = Line { text: Inked::plain(" "), hits: vec![], row: 0 };
    steps(s, &mut bar);
    next(s, th, &mut bar, w);
    let mut row = Line { text: Inked::new(), hits: vec![], row: 1 };
    kinds_row(s, th, &mut row, w);
    let mut hits = bar.hits;
    hits.extend(row.hits);
    let mut out = vec![bar.text.cut(w), row.text.clone().cut(w)];
    let room = w as i64 - row.text.width() as i64 - 3;
    if tally.is_empty() { return (out, hits); }
    if room >= crate::text::width(tally) as i64 {
        let gap = w - row.text.width() - crate::text::width(tally) - 1;
        out[1] = row.text.add(Inked::plain(" ".repeat(gap))).add(Inked::of(tally, Ink::Muted));
    } else { out.push(Inked::plain(" ").add(Inked::of(crate::text::cut(tally, w.saturating_sub(1).max(1)), Ink::Muted))); }
    (out, hits)
}
