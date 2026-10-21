//! The filter row on top: All · Needs you · Working · Ready · To do · Done · Coordinators, each with its count. Tab and
//! shift+tab go round it, a click picks one, the list shows only that status; Coordinators lists every coordinator and
//! enter or space on one talks to it. Alt+S groups by status, by repo, by machine, and round again.
use crate::common;
use common::board::open;
use common::*;
use serde_json::json;
use sw::board::state::Row;
use sw::board::update::{update, Event};
use sw::json::*;

fn coord_world(machines: &str) -> World {
    let w = world_with(Opts { repo: true, settings: json!({ "coordinator.agent": "claude", "review.when": "never", "board.machines": machines }), ..Default::default() });
    let home = w.base.join("home-coord");
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("HERDR_PROJECTS_COORDINATOR_DIR", &home) };
    w
}
fn drain() { flush(); sw::runner::drain_jobs(&|job, id, extra| { sw::jobs::run_job(job, id, extra); flush(); }); }
fn tasks_shown(s: &common::board::Board) -> Vec<String> { s.s.rows.iter().filter_map(|r| match r { Row::Rec(o) if !o.has("coord") => Some(o.t("title")), _ => None }).collect() }
fn coords_shown(s: &common::board::Board) -> Vec<String> { s.s.rows.iter().filter_map(|r| match r { Row::Rec(o) if o.has("coord") => Some(o.t("title")), Row::Head { name, .. } => Some(format!("▾ {name}")), _ => None }).collect() }
fn heads(s: &common::board::Board) -> Vec<String> { s.s.rows.iter().filter_map(|r| match r { Row::Head { name, .. } => Some(name.clone()), _ => None }).collect() }
fn to_all(s: &mut common::board::Board) { for _ in 0..4 { if s.s.scope == "*" { return; } s.press(&["alt+m"]); } panic!("never every machine"); }

#[test]
fn tab_goes_round_the_filters_and_the_list_shows_only_that_status() {
    let w = coord_world("none");
    let asks = w.add("Asks you"); sw::actions::filing::report(&asks, sw::actions::filing::Report { state: "blocked".into(), msg: "which one?".into(), ..Default::default() }).unwrap();
    w.add("Plain to do");
    for n in 0..7 { let id = w.add(&format!("Done {n}")); sw::store::tasks::update(&id, |t| { t.put("stage", "done"); Ok(sw::store::tasks::Change::Write) }).unwrap(); }
    let mut s = open(&w.root, 40); s.s.side = false;
    let l = s.screen(120);
    assert_eq!(l[0], " herdr-projects  proj · PRJ");
    // a coordinator's row reads as a task agent's: its mark in the marks' column, no heading's rule
    assert!(l.iter().any(|x| regex::Regex::new(r"^ · +Main coordinator$").unwrap().is_match(x)) && !l.iter().any(|x| x.starts_with(" ") && x.contains("──")), "{l:?}");
    assert_eq!(l[1], " All 9 · Needs you 1 · Working 0 · Ready 0 · To do 1 · Done 7 · Coordinators 3");
    // tab: needs you only
    s.press(&["tab"]); assert_eq!(s.s.show, 1);
    assert_eq!(tasks_shown(&s), vec!["Asks you"]);
    assert!(!s.s.rows.iter().any(|r| matches!(r, Row::Rec(o) if o.has("coord"))), "no coordinator's row but under All and Coordinators");
    s.press(&["tab", "tab", "tab"]); assert_eq!(s.s.show, 4); assert_eq!(tasks_shown(&s), vec!["Plain to do"]);
    // done: every done task, not the latest five
    s.press(&["tab"]); assert_eq!(tasks_shown(&s).len(), 7, "{:?}", tasks_shown(&s));
    // shift+tab: back; round from All to Coordinators
    s.press(&["shift+tab"]); assert_eq!(s.s.show, 4);
    s.press(&["tab", "tab", "tab"]); assert_eq!(s.s.show, 0);
    s.press(&["shift+tab"]); assert_eq!(s.s.show, 6);
    // a click on a filter picks it
    s.lines();
    let (y, x, _, _) = *s.s.show_hits.iter().find(|h| h.3 == 1).unwrap();
    let fx = update(&mut s.s, Event::Click { x: x as u16, y: y as u16 }); s.run(fx);
    assert_eq!(s.s.show, 1);
    // tab never puts an agent on a task any more
    assert!(sw::store::tasks::list(Default::default()).iter().all(|t| t.s("stage") != "progress"));
    // kept with the view
    let view: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(sw::store::files::data_dir().join("view.json")).unwrap()).unwrap();
    assert_eq!(view.at("show"), &json!(1));
    // narrow: shorter names, then only the one shown
    assert!(s.screen(72)[1].starts_with(" All 9 · Needs 1 · Working 0"), "{:?}", s.screen(72));
    assert_eq!(s.screen(30)[1], " Needs you 1  tab next");
}

#[test]
fn coordinators_is_a_filter_its_rows_stand_where_the_grouping_puts_them_for_the_machine_shown() {
    let w = coord_world("otherbox");
    w.add("Fix the login");
    w.script(json!({ "coordinator": [{ "said": "Proj is quiet." }] }));
    let host = sw::shown::words::clean(&sw::remote::check::hostname());
    let mut s = open(&w.root, 40); s.s.side = false;
    // under All: the main Coordinator's row only, as a task agent's row
    assert_eq!(coords_shown(&s), vec!["Main coordinator", "▾ to do"]);
    // Coordinators, this machine: its main, its machine's (never ran) and its project's (used lately)
    s.press(&["shift+tab"]); assert_eq!(s.s.show, 6);
    assert_eq!(coords_shown(&s), vec!["Main coordinator", "Machine coordinator", "Project coordinator · proj"]);
    assert!(tasks_shown(&s).is_empty());
    let t = s.text();
    assert!(t.contains("not started: your first message starts it") && !t.lines().any(|x| x.starts_with(" ") && x.contains("──")), "{t}");
    // enter on one: the chat talks to it; what you type goes to it
    s.s.sel = sw::board::update::coord_rows::row_id("", "project:proj"); s.load();
    assert!(s.text().contains("enter talk to the proj coordinator"), "{}", s.text());
    s.press(&["enter"]); assert!(s.text().contains("Send message to proj coordinator"), "{}", s.text());
    s.typed("hello"); s.press(&["enter"]); drain(); s.load();
    assert!(s.text().contains("Proj is quiet."), "{}", s.text());
    s.press(&["space"]);
    // by repo: the project's under its repo's heading
    s.press(&["alt+s"]);
    assert_eq!(coords_shown(&s), vec!["Main coordinator", "Machine coordinator", "▾ proj", "Project coordinator · proj"]);
    s.press(&["alt+s", "alt+s"]);
    // every machine: each machine's coordinators under its heading; the count follows the machine shown
    update(&mut s.s, Event::Machines(sw::remote::machines()));
    let answer = json!({ "host": "otherbox", "version": "9.0.0", "coordinator": { "state": "working", "readable": true, "said": "Far main is busy." },
        "coordinators": { "machine": { "state": "idle", "said": "Far machine is quiet." }, "project:far": { "state": "idle", "said": "Far project." } }, "tasks": [] });
    update(&mut s.s, Event::Remote { label: "otherbox".into(), answer: Ok(answer) });
    to_all(&mut s);
    let mut want: Vec<String> = vec![format!("▾ {host}"), "Main coordinator".into(), "Machine coordinator".into(), "Project coordinator · proj".into(), "▾ otherbox".into(), "Main coordinator".into(), "Machine coordinator".into(), "Project coordinator · far".into()];
    if sw::board::update::remotes::place(&s.s, "otherbox") < sw::board::update::remotes::place(&s.s, "") { want.rotate_left(4); }
    assert_eq!(coords_shown(&s), want);
    assert!(s.screen(140)[1].starts_with(" All machines ▾ │ All 1 · ") && s.screen(140)[1].ends_with("Coordinators 6"), "{:?}", s.screen(140));
    let t = s.text();
    assert!(t.contains("Far main is busy.") && t.contains("Far project."), "{t}");
    // one machine shown: its coordinators only, and the count is its
    s.press(&["alt+m"]); while s.s.scope != "otherbox" { s.press(&["alt+m"]); }
    assert_eq!(coords_shown(&s), vec!["Main coordinator", "Machine coordinator", "Project coordinator · far"]);
    assert!(s.screen(140)[1].starts_with(" otherbox ▾ │ ") && s.screen(140)[1].ends_with("Coordinators 3"), "{:?}", s.screen(140));
    // enter on its main Coordinator: what you type goes to that machine's
    s.s.sel = sw::board::update::coord_rows::row_id("otherbox", ""); s.load();
    s.press(&["enter"]);
    assert!(s.text().contains("Send message to Coordinator on otherbox"), "{}", s.text());
    s.typed("hello far");
    let fx = s.hold("enter");
    let jobs: Vec<String> = fx.iter().filter_map(|e| match e { sw::board::update::Effect::Run(j) => Some(format!("{j:?}")), _ => None }).collect();
    assert!(jobs.iter().any(|j| j.starts_with("Dispatch") && j.contains("hello far") && j.contains("otherbox")), "{jobs:?}");
    // a click on the machine item goes round as alt+m does
    s.lines();
    let (y, x, _, _) = *s.s.show_hits.iter().find(|h| h.3 == sw::board::view::shows_row::MACHINE).unwrap();
    let was = s.s.scope.clone();
    let fx = update(&mut s.s, Event::Click { x: x as u16, y: y as u16 }); s.run(fx);
    assert_ne!(s.s.scope, was);
}

#[test]
fn alt_s_groups_by_status_by_repo_by_machine_and_round_again() {
    let w = coord_world("none");
    w.add("One"); w.add("Two");
    let host = sw::shown::words::clean(&sw::remote::check::hostname());
    let mut s = open(&w.root, 40); s.s.side = false;
    assert_eq!(heads(&s), vec!["to do"]);
    s.press(&["alt+s"]); assert!(s.s.by_repo); assert_eq!(heads(&s), vec!["proj"]);
    s.press(&["alt+s"]); assert!(s.s.by_machine && !s.s.by_repo);
    assert_eq!(heads(&s), vec![host.clone()], "the machine's heading alone, no status heading under it");
    assert_eq!(tasks_shown(&s).len(), 2);
    s.press(&["alt+s"]); assert!(!s.s.by_machine && !s.s.by_repo); assert_eq!(heads(&s), vec!["to do"]);
}
