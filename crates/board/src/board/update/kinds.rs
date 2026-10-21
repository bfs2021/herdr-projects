//! The kinds row under the scope bar: all · tasks · agents · coordinators · automations. The kind picked narrows the
//! list to its rows and stays while the scope changes. Coordinators lists the scope's coordinators (on All: every
//! coordinator of every machine), each as a row with where it stands and its last line; enter on one goes into its
//! scope (scope.rs: jump).
use super::coords::of_head;
use super::heads::machine_id;
use super::list::relist;
use super::Effect;
use crate::board::keys::{is, Press, K};
use crate::board::state::{Row, State};
use crate::json::*;
use crate::shown::states::is_live;

pub const KINDS: [&str; 5] = ["all", "tasks", "agents", "coordinators", "automations"];
pub const ALL: usize = 0;
pub const TASKS: usize = 1;
pub const AGENTS: usize = 2;
pub const COORDS: usize = 3;
pub const AUTOS: usize = 4;

/// The list is the list of coordinators (not in the archive, which holds tasks only).
pub fn coords(s: &State) -> bool { s.kind == COORDS && !s.archive_view }
/// A task stays in the list of this kind: every task, or only those an agent is on (working, waiting for you, ready).
pub fn keeps_task(s: &State, r: &Obj) -> bool { match s.kind { ALL | TASKS => true, AGENTS => is_live(r) || s.archive_view, _ => s.archive_view } }
/// Automations stand in the list: under all, and alone under automations.
pub fn keeps_autos(s: &State) -> bool { matches!(s.kind, ALL | AUTOS) }
/// The main Coordinator's row on top of the list: under all only (coordinators has its own list).
pub fn keeps_main(s: &State) -> bool { s.kind == ALL }
pub fn set(s: &mut State, kind: usize, fx: &mut Vec<Effect>) {
    s.kind = kind.min(KINDS.len() - 1);
    fx.push(Effect::Save);
    relist(s);
}
/// The keys while the kinds row has them: the left and right arrows choose, up stays, enter, down, esc or the kinds
/// key again go back to the list. Any other key goes back to the list and does what it always does there.
pub fn key(s: &mut State, p: &Press, fx: &mut Vec<Effect>) -> bool {
    if is(p, K::ScopeOut) { let k = s.kind.checked_sub(1).unwrap_or(KINDS.len() - 1); set(s, k, fx); return true; }
    if is(p, K::ScopeIn) { let k = (s.kind + 1) % KINDS.len(); set(s, k, fx); return true; }
    if is(p, K::Up) { return true; }
    s.kinds_focus = false;
    is(p, K::Enter) || is(p, K::Down) || is(p, K::Back) || is(p, K::Kinds)
}
/// The projects of a machine ("" this one) that have a project coordinator, as far as the board knows.
fn projects(s: &State, host: &str) -> Vec<String> {
    let keys: Vec<String> = if host.is_empty() { s.data.looks.keys().cloned().collect() } else { s.remotes.get(host).and_then(|m| m.coords.as_object().map(|o| o.keys().cloned().collect())).unwrap_or_default() };
    let mut out: Vec<String> = keys.iter().filter_map(|k| k.strip_prefix("project:").map(String::from)).collect();
    out.sort_by_key(|r| r.to_lowercase());
    out
}
/// The coordinators of the scope, as rows: on All the Coordinator, then for each machine (that answered) its own main
/// Coordinator (another machine's), its machine coordinator and its projects'; on a machine its own and its projects'; on a project only its own.
pub fn coord_rows(s: &State) -> Vec<Row> {
    let head = |name: String, id: String| Row::Head { name, autos: false, id, machine: false, folded: 0, folded_autos: 0, empty: false };
    let named = |id: String| of_head(s, &id).map(|t| head(t.name, id));
    let mut out = vec![];
    if s.level >= 2 { return named(super::scope::repo_id(&s.scope, &s.project)).into_iter().collect(); }
    let hosts: Vec<String> = if s.level == 0 {
        if s.data.can_dispatch { out.extend(named(super::pilot::main_id(""))); }
        let mut h: Vec<String> = vec![String::new()];
        h.extend(super::remotes::others(s).into_iter().filter(|m| m.ok).map(|m| m.machine.label.clone()));
        h
    } else { vec![s.scope.clone()] };
    for h in hosts {
        // Another machine's own main Coordinator, on All, where it sent one.
        if s.level == 0 && !h.is_empty() && s.remotes.get(&h).and_then(|m| crate::coordinator::far::read(&m.coord)).is_some() { out.extend(named(super::pilot::main_id(&h))); }
        out.extend(named(machine_id(&h)));
        for r in projects(s, &h) { out.extend(named(super::scope::repo_id(&h, &r))); }
    }
    out
}
