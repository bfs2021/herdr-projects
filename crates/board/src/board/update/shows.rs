//! The filter row on top of the board: All · Needs you · Working · Ready · To do · Done · Coordinators, each with its
//! count. Tab moves to the next, shift+tab back, a click picks one; the list then shows only the tasks of that status.
//! Coordinators lists every coordinator, the main one, each machine's and each project's, on every machine that
//! answered, each as a row with where it stands and its last line; enter or space on one talks to it.
use super::coords::of_head;
use super::heads::{machine_id, HEAD};
use super::list::relist;
use super::Effect;
use crate::board::keys::Press;
use crate::board::state::{Row, State};
use crate::json::*;
use crate::shown::states::group_of;

/// The filters, each with the status group it shows (None: every task, or the coordinators).
pub const SHOWS: [(&str, Option<&str>); 7] = [("All", None), ("Needs you", Some("needs you")), ("Working", Some("working")), ("Ready", Some("ready to approve")), ("To do", Some("to do")), ("Done", Some("done")), ("Coordinators", None)];
pub const COORDS: usize = 6;

/// The list is the list of coordinators (not in the archive, which holds tasks only).
pub fn coords(s: &State) -> bool { s.show == COORDS && !s.archive_view }
fn group(s: &State) -> Option<&'static str> { if s.archive_view { None } else { SHOWS.get(s.show).and_then(|x| x.1) } }
/// A task stays in the list of this filter: every task, or those of its status.
pub fn keeps_task(s: &State, r: &Obj) -> bool { !coords(s) && group(s).is_none_or(|g| group_of(r) == g) }
/// Automations and the main Coordinator's row stand in the list under All only.
pub fn keeps_all(s: &State) -> bool { s.show == 0 || s.archive_view }
/// Done shows every done task, not only the latest few.
pub fn whole(s: &State) -> bool { group(s) == Some("done") }
/// How many each filter holds, from the tasks in view (archived ones left out) and the coordinators.
pub fn counts(s: &State, items: &[std::sync::Arc<Obj>]) -> Vec<usize> {
    let live: Vec<&std::sync::Arc<Obj>> = items.iter().filter(|r| !r.b("archivedAt")).collect();
    SHOWS.iter().enumerate().map(|(n, (_, g))| match g { Some(g) => live.iter().filter(|r| group_of(r) == *g).count(), None if n == COORDS => coord_rows(s).len(), None => live.len() }).collect()
}
/// Tab: the next filter; shift+tab the one before.
pub fn step(s: &mut State, p: &Press, fx: &mut Vec<Effect>) {
    let n = SHOWS.len();
    let to = if p.chord.starts_with("shift+") { (s.show + n - 1) % n } else { (s.show + 1) % n };
    set(s, to, fx);
}
/// Show the filter `n`. The coordinators of other machines come with their tasks: they are asked at once.
pub fn set(s: &mut State, n: usize, fx: &mut Vec<Effect>) {
    s.show = n.min(SHOWS.len() - 1);
    fx.push(Effect::Save);
    if coords(s) { fx.push(Effect::Poll(true)); }
    relist(s);
}
/// Enter or space on a coordinator in the list: talk to it. The main Coordinator is who the chat talks to anyway.
pub fn talk(s: &mut State, id: &str) {
    if *id == super::pilot::main_id("") { s.reply = false; return; }
    s.sel = id.to_string();
    super::coords::flip(s);
}
/// The projects of a machine ("" this one) that have a project coordinator, as far as the board knows.
fn projects(s: &State, host: &str) -> Vec<String> {
    let keys: Vec<String> = if host.is_empty() { s.data.looks.keys().cloned().collect() } else { s.remotes.get(host).and_then(|m| m.coords.as_object().map(|o| o.keys().cloned().collect())).unwrap_or_default() };
    let mut out: Vec<String> = keys.iter().filter_map(|k| k.strip_prefix("project:").map(String::from)).collect();
    out.sort_by_key(|r| r.to_lowercase());
    out
}
/// Every coordinator, as rows: this machine's main Coordinator, its machine coordinator and its projects' that were
/// ever talked to; then each other machine that answered, in the order of herdr's sidebar: its own main Coordinator
/// (where it sent one), its machine coordinator and its projects'.
pub fn coord_rows(s: &State) -> Vec<Row> {
    let named = |id: String| of_head(s, &id).map(|t| Row::Head { name: t.name, autos: false, id, machine: false, folded: 0, folded_autos: 0, empty: false });
    let mut out = vec![];
    if s.data.can_dispatch { out.extend(named(super::pilot::main_id(""))); }
    let mut hosts: Vec<String> = vec![String::new()];
    hosts.extend(super::remotes::others(s).into_iter().filter(|m| m.ok).map(|m| m.machine.label.clone()));
    for h in hosts {
        if !h.is_empty() && s.remotes.get(&h).and_then(|m| crate::coordinator::far::read(&m.coord)).is_some() { out.extend(named(super::pilot::main_id(&h))); }
        out.extend(named(machine_id(&h)));
        for r in projects(s, &h) { out.extend(named(format!("{HEAD}repo\t{h}\t{r}"))); }
    }
    out
}
