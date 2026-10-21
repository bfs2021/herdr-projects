//! The coordinators as rows of the list, in a task agent's grammar: a mark for where each stands (the ones the chat's
//! line has for a coordinator or an agent, `◎` on autopilot), its name saying what it is (`Main coordinator`, `Machine
//! coordinator`, `Project coordinator · herdr-projects`), the last thing it said under it, and its time on the right.
//! Each stands where the grouping puts it: the main and the machine's at the top of its machine's part, a project's at
//! the top of its repo's group by repo, else with them. Under All only those that were ever talked to; under the filter
//! Coordinators every one, the machine's and the projects' used lately that never ran too, so that a first message
//! starts them. A row is a record `{id, coord}` (view/fields.rs reads it), its id a heading's kind (heads.rs: HEAD), so
//! the selection, space, the start key, `Ctrl+X`, `Ctrl+E` and the panel take it as a coordinator.
use super::coords::{look_of, of_head, Target};
use super::heads::HEAD;
use crate::board::state::{Rec, State};
use crate::json::*;

/// A coordinator's row id: its machine ("" this one) and its level ("" the main one, "machine", "project:<repo>").
pub fn row_id(host: &str, to: &str) -> String {
    match to { "" => super::pilot::main_id(host), "machine" => format!("{HEAD}cm\t{host}"), t => format!("{HEAD}cp\t{host}\t{}", t.strip_prefix("project:").unwrap_or(t)) }
}
/// What a row calls it: what it is, the machine left to its heading (or the filter row).
pub fn row_name(t: &Target) -> String {
    match t.to.strip_prefix("project:") { Some(r) => format!("Project coordinator · {}", crate::shown::words::repo_label(r)), None if t.to == "machine" => "Machine coordinator".into(), None => "Main coordinator".into() }
}
/// The levels of a machine's coordinators that were ever talked to ("machine", "project:<repo>").
fn known(s: &State, host: &str) -> Vec<String> {
    if host.is_empty() { return s.data.coords.clone(); }
    s.remotes.get(host).and_then(|m| m.coords.as_object().map(|o| o.keys().cloned().collect())).unwrap_or_default()
}
/// A machine's main Coordinator is there: this machine's where one can run, another's where it sent one.
fn has_main(s: &State, host: &str) -> bool {
    if host.is_empty() { s.data.can_dispatch } else { s.remotes.get(host).and_then(|m| crate::coordinator::far::read(&m.coord)).is_some() }
}
/// A machine's coordinators' rows in view: its main and its machine coordinator first, then each project's with its repo.
pub fn of_machine(s: &State, host: &str) -> (Vec<String>, Vec<(String, String)>) {
    if !super::shows::keeps_coords(s) { return (vec![], vec![]); }
    of_machine_as(s, host, super::shows::coords(s))
}
/// The same, `every` one (the filter Coordinators) or only those that were ever talked to.
fn of_machine_as(s: &State, host: &str, every: bool) -> (Vec<String>, Vec<(String, String)>) {
    let known = known(s, host);
    let mut top: Vec<String> = vec![];
    if has_main(s, host) { top.push(row_id(host, "")); }
    if every || known.iter().any(|k| k == "machine") { top.push(row_id(host, "machine")); }
    let mut repos: Vec<String> = known.iter().filter_map(|k| k.strip_prefix("project:").map(String::from)).collect();
    if every && host.is_empty() { repos.extend(s.data.lately.iter().cloned()); }
    repos.sort_by_key(|r| r.to_lowercase()); repos.dedup();
    (top, repos.into_iter().map(|r| { let id = row_id(host, &format!("project:{r}")); (r, id) }).collect())
}
/// How many coordinator rows the filter Coordinators shows for whose board is in view.
pub fn count(s: &State, hosts: &[String]) -> usize { hosts.iter().map(|h| { let (top, repos) = of_machine_as(s, h, true); top.len() + repos.len() }).sum() }
/// Where a coordinator stands, as the chat's line marks it: its mark's kind and its word (`◎` on autopilot).
fn stands(s: &State, t: &Target, main_here: bool) -> (&'static str, &'static str, String) {
    if t.host.is_empty() && !s.data.can_dispatch { return ("unknown", "off here", String::new()); }
    let p = super::pilot::view(s, t);
    if p.b("on") { return ("pilot", "on autopilot", p.t("state")); }
    let v = if main_here { Some(s.data.coord.clone()) } else { look_of(s, t) };
    let (state, err) = v.as_ref().map(|v| (v.state.clone(), !v.error.is_empty())).unwrap_or(("none".into(), false));
    if err { return ("error", "needs you", String::new()); }
    match state.as_str() { "working" | "starting" => ("working", "working", String::new()), "blocked" => ("blocked", "needs you", String::new()), "idle" => ("idle", "idle", String::new()), "exited" => ("unknown", "stopped", String::new()), _ => ("unknown", "not started", String::new()) }
}
/// A coordinator's row, as a record the list draws as a task agent's (view/fields.rs: is_coord). `title` and `state` are
/// there for the web board's row too.
pub fn rec(s: &State, id: &str) -> Option<Rec> {
    let t = of_head(s, id)?;
    let main_here = *id == row_id("", "");
    let (kind, word, pilot) = stands(s, &t, main_here);
    let v = if main_here { Some(s.data.coord.clone()) } else { look_of(s, &t) };
    let last = v.as_ref().and_then(|v| v.talk.last().cloned());
    let said = v.as_ref().map(|v| if !v.said.trim().is_empty() { v.said.clone() } else { last.as_ref().map(|x| x.text.clone()).unwrap_or_default() }).unwrap_or_default();
    // On autopilot: what it does now under its name, and its timing where a task has its columns.
    let p = super::pilot::view(s, &t);
    let said = if p.b("on") { if p.b("activity") { p.t("activity") } else { p.t("line") } } else { said };
    let line = said.lines().map(str::trim).find(|l| !l.is_empty()).map(crate::shown::words::clean).unwrap_or_else(|| match word { "not started" => "not started: your first message starts it".into(), "off here" => crate::coordinator::who::OFF_WORDS.into(), _ => "nothing said yet".into() });
    let at = last.map(|x| x.at).unwrap_or(0);
    let state = match kind { "working" => "working", "blocked" | "error" => "blocked", _ => "idle" };
    Some(Rec::new(obj(json!({ "id": id, "title": row_name(&t), "state": state, "msg": line, "host": t.host,
        "coord": { "kind": kind, "word": word, "pilot": pilot, "said": line, "at": at, "main": t.to.is_empty(),
            "added": if p.b("on") { p.t("added") } else { String::new() }, "last": if p.b("on") && p.b("last") { format!("last {}", p.s("last")) } else { String::new() }, "next": if p.b("on") { p.t("next") } else { String::new() } } }))))
}
