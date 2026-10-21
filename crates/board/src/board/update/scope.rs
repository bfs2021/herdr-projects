//! The scope bar from the board's side: All › a machine › one of its projects. Each step is a scope, which decides what
//! the list shows and who the chat talks to: All every machine and the Coordinator, a machine its tasks and its machine
//! coordinator, a project its tasks and its project coordinator. The right arrow steps into the next level through a
//! picker that takes typed letters (fuzzy: in order, not side by side); the left arrow goes back up a level.
use super::coords::{of_head, Target};
use super::heads::{host_of, machine_id, HEAD};
use super::list::relist;
use super::Effect;
use crate::board::keys::{is, key_name, Press, K};
use crate::board::state::{Pick, State};
use crate::json::*;
use crate::shown::ink::Ink;

/// A project's step, as a heading's id: its project coordinator's (coords.rs: of_head).
pub fn repo_id(host: &str, repo: &str) -> String { format!("{HEAD}repo\t{host}\t{repo}") }
/// The coordinator of the scope the bar stands on: None on All, which talks to the Coordinator as ever.
/// This machine's, only where a coordinator can run here: else what you type is added as a task, as on All.
pub fn coord(s: &State) -> Option<Target> {
    let t = match s.level { 0 => None, 1 => of_head(s, &machine_id(&s.scope)), _ => of_head(s, &repo_id(&s.scope, &s.project)) };
    t.filter(|t| !t.host.is_empty() || s.data.can_dispatch)
}
/// The level kept in step with whose board shows, however that changed (the web board picks it, Alt+M goes round): every
/// machine is All, one machine by name is that machine; a project stays only on its own machine.
pub fn settle(s: &mut State) {
    if s.scope == "*" { s.level = 0; } else if !s.scope.is_empty() && s.level == 0 { s.level = 1; }
    if s.level >= 2 && (s.project.is_empty() || s.project_on != s.scope) { s.level = 1; }
    if s.level < 2 { s.project.clear(); s.project_on.clear(); }
}
/// A task's repo as its project's step names it: another machine's by its own id there.
pub fn repo_of(r: &Obj) -> String { (if r.b("rid") { r.s("rid") } else { r.s("id") }).split('/').next().unwrap_or("").to_string() }
/// Where the steps go next: on All the machines, in the order of herdr's sidebar; on a machine its projects (those with
/// tasks or automations, those used lately, those with a project coordinator), the board's own repo first.
pub fn candidates(s: &State) -> Vec<(String, String)> {
    if s.level == 0 {
        let mut hosts: Vec<String> = vec![String::new()];
        hosts.extend(super::remotes::others(s).into_iter().map(|m| m.machine.label.clone()));
        hosts.sort_by_key(|h| super::remotes::place(s, h));
        return hosts.into_iter().map(|h| { let n = super::remotes::shown(s, &h); (h, n) }).collect();
    }
    if s.level >= 2 { return vec![]; }
    let here = s.scope.is_empty();
    let (items, autos, coords): (Vec<Obj>, Vec<Obj>, Vec<String>) = if here { (s.data.items.iter().map(|r| (**r).clone()).collect(), s.data.autos.clone(), s.data.looks.keys().cloned().collect()) }
        else { s.remotes.get(&s.scope).map(|m| (m.items.iter().map(|r| (**r).clone()).collect(), m.autos.clone(), m.coords.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default())).unwrap_or_default() };
    let mut names: Vec<String> = items.iter().filter(|r| !r.b("archivedAt")).map(repo_of).collect();
    names.extend(autos.iter().map(|a| a.t("repo")));
    names.extend(coords.iter().filter_map(|c| c.strip_prefix("project:").map(String::from)));
    if here { names.extend(s.data.lately.iter().cloned()); }
    names.retain(|n| !n.is_empty());
    names.sort_by_key(|n| (!(here && *n == s.data.repo), n.to_lowercase()));
    names.dedup();
    names.into_iter().map(|n| { let l = crate::shown::words::repo_label(&n); (n, l) }).collect()
}
/// How well typed letters name a step: None when they are not all in it in order. Lower is better: its start, then
/// anywhere in one piece, then spread out (the fewer cells between them the better).
pub fn fuzzy(typed: &str, name: &str) -> Option<(u8, usize)> {
    let (q, n) = (typed.to_lowercase(), name.to_lowercase());
    if q.is_empty() { return Some((0, 0)); }
    if n.starts_with(&q) { return Some((0, 0)); }
    if let Some(at) = n.find(&q) { return Some((1, at)); }
    let (mut gaps, mut last, mut it) = (0, None, n.char_indices());
    for c in q.chars() {
        let (at, _) = it.by_ref().find(|(_, x)| *x == c)?;
        if let Some(l) = last { gaps += at - l - 1; }
        last = Some(at);
    }
    Some((2, gaps))
}
/// The steps the picker offers for what is typed, best first.
pub fn matches(s: &State) -> Vec<(String, String)> {
    let typed = s.pick.as_ref().map(|p| p.typed.clone()).unwrap_or_default();
    let mut all: Vec<((u8, usize), usize, (String, String))> = candidates(s).into_iter().enumerate().filter_map(|(n, c)| fuzzy(&typed, &c.1).map(|f| (f, n, c))).collect();
    all.sort_by_key(|(f, n, _)| (*f, *n));
    all.into_iter().map(|(_, _, c)| c).collect()
}
/// Right arrow: the picker for the next level, on the selected row's machine or project. With one machine only, All
/// steps into it at once.
pub fn step_in(s: &mut State, fx: &mut Vec<Effect>) {
    if s.level >= 2 { return s.flash(format!("A project is the last step. {} goes back up.", key_name("scopeOut")), Ink::Muted); }
    let c = candidates(s);
    if c.is_empty() { return s.flash(format!("No projects on {} yet.", super::remotes::shown(s, &s.scope)), Ink::Muted); }
    if c.len() == 1 && s.level == 0 { let h = c[0].0.clone(); return go(s, 1, &h, "", fx); }
    let r = s.current().cloned();
    let near = if s.level == 0 { r.as_ref().map(host_of) } else { r.as_ref().map(repo_of) };
    let at = near.and_then(|n| c.iter().position(|x| x.0 == n)).unwrap_or(0);
    s.kinds_focus = false;
    s.pick = Some(Pick { typed: String::new(), at });
}
/// Left arrow: up a level.
pub fn step_out(s: &mut State, fx: &mut Vec<Effect>) {
    match s.level {
        0 => s.flash(format!("All is the first step. {} steps into a machine.", key_name("scopeIn")), Ink::Muted),
        1 => go(s, 0, "", "", fx),
        _ => { let h = s.scope.clone(); go(s, 1, &h, "", fx) }
    }
}
/// The keys while the picker is open: letters pick, the arrows go through the matches, enter picks, esc cancels.
pub fn pick_key(s: &mut State, p: &Press, fx: &mut Vec<Effect>) -> bool {
    let Some(mut pk) = s.pick.clone() else { return false };
    if is(p, K::Close) || is(p, K::Focus) { s.pick = None; return false; }
    if is(p, K::Back) || is(p, K::ScopeOut) { s.pick = None; return true; }
    let found = matches(s);
    if is(p, K::Enter) || is(p, K::ScopeIn) {
        let Some((id, _)) = found.get(pk.at.min(found.len().saturating_sub(1))).cloned() else { s.flash(format!("Nothing matches \"{}\".", pk.typed), Ink::Yellow); return true };
        let host = s.scope.clone();
        if s.level == 0 { go(s, 1, &id, "", fx); } else { go(s, 2, &host, &id, fx); }
        return true;
    }
    if is(p, K::Up) { pk.at = pk.at.saturating_sub(1); }
    else if is(p, K::Down) { pk.at = (pk.at + 1).min(found.len().saturating_sub(1)); }
    else if p.chord == "backspace" { pk.typed.pop(); pk.at = 0; }
    else if let Some(c) = p.ch.filter(|c| !c.is_control()) { pk.typed.push(c); pk.at = 0; }
    s.pick = Some(pk);
    true
}
/// Go to a scope: its level, its machine ("" this one) and its project. The chat goes to its coordinator.
pub fn go(s: &mut State, level: u8, host: &str, project: &str, fx: &mut Vec<Effect>) {
    s.pick = None; s.kinds_focus = false;
    s.level = level;
    s.scope = if level == 0 { if s.remotes.is_empty() { String::new() } else { "*".into() } } else { host.to_string() };
    s.project = if level >= 2 { project.to_string() } else { String::new() };
    s.project_on = if level >= 2 { host.to_string() } else { String::new() };
    s.reply = false; s.full = false; s.panel_top = 0;
    fx.push(Effect::Save);
    if s.all() { fx.push(Effect::Poll(true)); }
    let who = coord(s).map(|t| t.name).unwrap_or_else(|| crate::status_words::COORDINATOR.into());
    let what = match level { 0 => "All".to_string(), 1 => super::remotes::shown(s, host), _ => crate::shown::words::repo_label(project) };
    s.flash(format!("{what}: the chat talks to the {who}."), Ink::Muted);
    relist(s);
}
/// Enter on a coordinator in the list of coordinators: into its scope.
pub fn jump(s: &mut State, id: &str, fx: &mut Vec<Effect>) {
    let rest = id.strip_prefix(HEAD).unwrap_or("");
    match rest.split('\t').collect::<Vec<_>>().as_slice() {
        ["main", ""] => go(s, 0, "", "", fx),
        // Another machine's main Coordinator: into its machine (space on its row talks to it).
        ["main", host] => go(s, 1, host, "", fx),
        ["machine", host] => go(s, 1, host, "", fx),
        ["repo", host, repo] => go(s, 2, host, repo, fx),
        _ => {}
    }
}
/// A click on the scope bar or the kinds row: what view/scope_bar.rs put there.
pub fn click(s: &mut State, what: &str, fx: &mut Vec<Effect>) {
    let parts: Vec<&str> = what.split('\t').collect();
    match parts.as_slice() {
        ["level", "0"] => go(s, 0, "", "", fx),
        ["level", "1"] => { let h = s.scope.clone(); go(s, 1, &h, "", fx) }
        ["level", _] => {}
        ["machine", host] => go(s, 1, host, "", fx),
        ["project", repo] => { let h = s.scope.clone(); go(s, 2, &h, repo, fx) }
        ["more"] => step_in(s, fx),
        ["kind", n] => { s.pick = None; super::kinds::set(s, n.parse().unwrap_or(0), fx) }
        _ => {}
    }
}
