//! The filter row on top of the board: All · Needs you · Working · Ready · To do · Done · Coordinators, each with its
//! count. Tab moves to the next, shift+tab back, a click picks one; the list then shows only the tasks of that status.
//! Coordinators is a filter as the others are: the board with only the coordinators' rows left (coord_rows.rs), in the
//! same places and grouping, for the machine shown; enter or space on one talks to it.
use super::list::relist;
use super::Effect;
use crate::board::keys::Press;
use crate::board::state::State;
use crate::json::*;
use crate::shown::states::group_of;

/// The filters, each with the status group it shows (None: every task, or the coordinators).
pub const SHOWS: [(&str, Option<&str>); 7] = [("All", None), ("Needs you", Some("needs you")), ("Working", Some("working")), ("Ready", Some("ready to approve")), ("To do", Some("to do")), ("Done", Some("done")), ("Coordinators", None)];
pub const COORDS: usize = 6;

/// The filter Coordinators is on (not in the archive, which holds tasks only).
pub fn coords(s: &State) -> bool { s.show == COORDS && !s.archive_view }
fn group(s: &State) -> Option<&'static str> { if s.archive_view { None } else { SHOWS.get(s.show).and_then(|x| x.1) } }
/// A task stays in the list of this filter: every task, or those of its status.
pub fn keeps_task(s: &State, r: &Obj) -> bool { !coords(s) && group(s).is_none_or(|g| group_of(r) == g) }
/// Automations stand in the list under All only.
pub fn keeps_all(s: &State) -> bool { s.show == 0 || s.archive_view }
/// The coordinators' rows stand in the list under All and under Coordinators (not in the archive).
pub fn keeps_coords(s: &State) -> bool { !s.archive_view && (s.show == 0 || s.show == COORDS) }
/// Done shows every done task, not only the latest few.
pub fn whole(s: &State) -> bool { group(s) == Some("done") }
/// How many each filter holds, from the tasks in view (archived ones left out) and the coordinators.
pub fn counts(s: &State, items: &[std::sync::Arc<Obj>]) -> Vec<usize> {
    let live: Vec<&std::sync::Arc<Obj>> = items.iter().filter(|r| !r.b("archivedAt")).collect();
    SHOWS.iter().enumerate().map(|(n, (_, g))| match g { Some(g) => live.iter().filter(|r| group_of(r) == *g).count(), None if n == COORDS => super::coord_rows::count(s, &hosts(s)), None => live.len() }).collect()
}
/// Tab: the next filter; shift+tab the one before.
pub fn step(s: &mut State, p: &Press, fx: &mut Vec<Effect>) {
    let n = SHOWS.len();
    let to = if p.chord.starts_with("shift+") { (s.show + n - 1) % n } else { (s.show + 1) % n };
    set(s, to, fx);
}
/// Show the filter `n`.
pub fn set(s: &mut State, n: usize, fx: &mut Vec<Effect>) {
    s.show = n.min(SHOWS.len() - 1);
    fx.push(Effect::Save);
    relist(s);
}
/// Enter or space on a coordinator in the list: talk to it, by its row, whichever machine's board shows (this
/// machine's main Coordinator too: with one other machine's board shown, the chat's own Coordinator is that machine's).
pub fn talk(s: &mut State, id: &str) {
    s.sel = id.to_string();
    super::coords::flip(s);
}
/// The machines whose board is in view: this one, every one that answered, or the one shown.
pub fn hosts(s: &State) -> Vec<String> {
    if s.scope == "*" { let mut h = vec![String::new()]; h.extend(super::remotes::others(s).into_iter().filter(|m| m.ok).map(|m| m.machine.label.clone())); h.sort_by_key(|x| super::remotes::place(s, x)); h } else { vec![s.scope.clone()] }
}
