//! The scope bar on top of the board, All › machine › project: the right arrow steps in (a picker that takes typed
//! letters, enter picks, esc cancels), the left arrow goes back up, a click on a step goes there, and the scope decides
//! who the chat talks to. The kinds row under it narrows the list; coordinators lists every coordinator, enter on one
//! goes into its scope.
use crate::common;
use common::board::open;
use common::*;
use serde_json::{json, Value};
use sw::board::update::scope::fuzzy;
use sw::board::update::{update, Event};
use sw::json::*;

fn coord_world() -> World {
    let w = world_with(Opts { repo: true, settings: json!({ "coordinator.agent": "claude", "review.when": "never" }), ..Default::default() });
    let home = w.base.join("home-coord");
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("HERDR_PROJECTS_COORDINATOR_DIR", &home) };
    w
}
fn drain() { flush(); sw::runner::drain_jobs(&|job, id, extra| { sw::jobs::run_job(job, id, extra); flush(); }); }
fn prompts(w: &World, name: &str) -> Vec<String> { w.state().arr("panes").iter().find(|p| p.at("agent").s("name") == name).map(|p| p.at("agent").arr("prompts").iter().map(text).collect()).unwrap_or_default() }
fn click_on(s: &mut common::board::Board, what: &str) {
    s.lines(); // where each piece is, as last drawn
    let (y, x, _, _) = s.s.bar_hits.iter().find(|h| h.3 == what).cloned().unwrap_or_else(|| panic!("{what} is not on the bar: {:?}", s.s.bar_hits));
    let fx = update(&mut s.s, Event::Click { x: x as u16, y: y as u16 }); s.run(fx);
}

#[test]
fn letters_in_order_pick_a_step_its_start_first() {
    assert_eq!(fuzzy("her", "herdr-projects"), Some((0, 0)));
    assert_eq!(fuzzy("proj", "herdr-projects"), Some((1, 6)));
    assert!(fuzzy("hp", "herdr-projects").is_some_and(|f| f.0 == 2));
    assert_eq!(fuzzy("zq", "herdr-projects"), None);
    assert_eq!(fuzzy("", "anything"), Some((0, 0)));
}

#[test]
fn the_arrows_step_into_a_machine_and_a_project_and_the_chat_follows_the_scope() {
    let w = coord_world();
    w.add("Fix the login");
    w.script(json!({ "coordinator": [{ "said": "Proj is quiet." }] }));
    let host = sw::shown::words::clean(&sw::remote::check::hostname());
    let mut s = open(&w.root, 34);
    let l = s.screen(100);
    assert!(l[0].starts_with(" ○ All › ") && l[0].contains(&host), "{l:?}");
    assert!(l[1].starts_with(" all · tasks · agents · coordinators · automations") && l[2].starts_with(" proj · PRJ · 1 to do"), "{l:?}");
    // with no side panel the counts stand on the kinds row's right
    s.press(&["alt+p"]);
    let l = s.screen(100);
    assert!(l[1].starts_with(" all · tasks · agents · coordinators · automations") && l[1].ends_with("proj · PRJ · 1 to do"), "{l:?}");
    // one machine only: the right arrow is into it at once, and the chat talks to its machine coordinator
    s.press(&["right"]);
    assert_eq!((s.s.level, s.s.scope.as_str()), (1, ""));
    let mc = format!("{host} coordinator");
    assert!(s.text().contains(&format!("❯ Send message to {mc}")), "{}", s.text());
    assert!(s.screen(100)[0].starts_with(&format!(" ○ All › ○ {host} › proj")), "{:?}", s.screen(100));
    // into a project: a picker; letters that are no project's match nothing, esc cancels
    s.press(&["right"]); assert!(s.s.pick.is_some());
    s.typed("zz"); assert!(s.screen(100)[0].contains("zz▏  nothing matches"), "{:?}", s.screen(100));
    s.press(&["esc"]); assert!(s.s.pick.is_none() && s.s.level == 1);
    s.press(&["right"]); s.typed("pj");
    assert!(s.text().contains("enter go in"), "{}", s.text());
    s.press(&["enter"]);
    assert_eq!((s.s.level, s.s.project.as_str()), (2, "proj"));
    assert!(s.screen(100)[0].starts_with(&format!(" ○ All › ○ {host} › ○ proj")), "{:?}", s.screen(100));
    assert!(s.text().contains("❯ Send message to proj coordinator"), "{}", s.text());
    // what you type goes to the project coordinator
    s.typed("what is open?"); s.press(&["enter"]); drain(); s.load();
    assert_eq!(prompts(&w, "sw-coordinator-proj"), vec!["what is open?"]);
    assert!(sw::coordinator::state::get(&w.socket).is_none(), "the Coordinator got nothing");
    // its dot: it runs now
    assert!(s.screen(100)[0].contains("● proj"), "{:?}", s.screen(100));
    // the left arrow: up a level, and up again to All, which talks to the Coordinator
    s.press(&["left"]); assert_eq!(s.s.level, 1);
    s.press(&["left"]); assert_eq!(s.s.level, 0);
    assert!(s.text().contains("❯ Send message to Coordinator"), "{}", s.text());
    s.press(&["left"]); assert!(s.s.msg.contains("All is the first step"), "{}", s.s.msg);
    // a click on a step goes there
    s.press(&["right"]); s.press(&["right"]); s.press(&["enter"]); assert_eq!(s.s.level, 2);
    click_on(&mut s, "level\t0"); assert_eq!(s.s.level, 0);
    click_on(&mut s, &format!("machine\t")); assert_eq!(s.s.level, 1);
    click_on(&mut s, "project\tproj"); assert_eq!((s.s.level, s.s.project.as_str()), (2, "proj"));
    // the scope is kept with the view
    let view: Value = serde_json::from_str(&std::fs::read_to_string(sw::store::files::data_dir().join("view.json")).unwrap()).unwrap();
    assert_eq!((view.at("level"), view.s("project")), (&json!(2), "proj"));
}

#[test]
fn the_kinds_row_narrows_the_list_and_coordinators_lists_each_one_enter_goes_into_its_scope() {
    let w = coord_world();
    w.add("Fix the login");
    w.script(json!({ "coordinator": [{ "said": "Proj is quiet." }] }));
    let mut s = open(&w.root, 34);
    // start the project coordinator, so it is one of the list
    s.press(&["right", "right", "enter"]); s.typed("hello"); s.press(&["enter"]); drain();
    s.press(&["left", "left"]); s.load();
    // the kinds row has the keys: the arrows choose
    s.press(&["shift+tab"]); assert!(s.s.kinds_focus);
    assert!(s.text().contains("→ ← choose a kind"), "{}", s.text());
    s.press(&["right"]); assert_eq!(s.s.kind, 1);
    s.press(&["right"]); assert_eq!(s.s.kind, 2);
    // agents: only tasks an agent is on (none here)
    assert!(!s.text().contains("Fix the login"), "{}", s.text());
    s.press(&["right"]); s.load(); s.load();
    let t = s.text();
    assert!(t.contains("Coordinator") && t.contains("proj coordinator") && t.contains("Proj is quiet."), "{t}");
    assert!(!t.contains("Fix the login"), "{t}");
    s.press(&["enter"]); assert!(!s.s.kinds_focus);
    // enter on a coordinator: into its scope; the kind stays
    s.s.sel = sw::board::update::scope::repo_id("", "proj"); s.load();
    s.press(&["enter"]);
    assert_eq!((s.s.level, s.s.project.as_str(), s.s.kind), (2, "proj", 3));
    // a click on a kind picks it
    click_on(&mut s, "kind\t0"); assert_eq!(s.s.kind, 0);
    assert!(s.text().contains("Fix the login"), "{}", s.text());
}

#[test]
fn all_with_coordinators_lists_another_machines_own_coordinators_too() {
    let w = world_with(Opts { repo: true, settings: json!({ "coordinator.agent": "claude", "review.when": "never", "board.machines": "otherbox" }), ..Default::default() });
    let mut s = open(&w.root, 34);
    update(&mut s.s, Event::Machines(sw::remote::machines()));
    let now = sw::clock::now();
    let answer = json!({ "host": "otherbox", "version": "9.0.0", "coordinator": { "state": "working", "readable": true, "said": "Far main is busy." },
        "coordinators": { "machine": { "state": "idle", "said": "Far machine is quiet." }, "project:far": { "state": "idle", "said": "Far project." } },
        "tasks": [{ "id": "far/1", "title": "Far away", "state": "idle", "stage": "todo", "at": now, "created": now, "snap": { "folder": "~/far" } }] });
    update(&mut s.s, Event::Remote { label: "otherbox".into(), answer: Ok(answer) });
    assert_eq!(s.s.scope, "*");
    s.press(&["shift+tab", "right", "right", "right", "enter"]); assert_eq!(s.s.kind, 3);
    let ids: Vec<String> = s.s.rows.iter().filter_map(|r| match r { sw::board::state::Row::Head { id, .. } => Some(id.clone()), _ => None }).collect();
    assert!(ids.contains(&sw::board::update::pilot::main_id("otherbox")), "{ids:?}");
    let t = s.text();
    assert!(t.contains("Coordinator on otherbox") && t.contains("Far main is busy.") && t.contains("otherbox coordinator") && t.contains("far coordinator · otherbox"), "{t}");
    // enter on it: into its machine
    s.s.sel = sw::board::update::pilot::main_id("otherbox"); s.load();
    s.press(&["enter"]); assert_eq!((s.s.level, s.s.scope.as_str()), (1, "otherbox"));
}

#[test]
fn a_picker_longer_than_the_bar_keeps_the_picked_one_in_view() {
    let w = coord_world();
    w.add("Fix the login");
    let mut s = open(&w.root, 34);
    s.press(&["right"]);
    for n in 0..20 { s.s.data.lately.push(format!("project-{n:02}")); }
    s.press(&["right"]);
    for _ in 0..19 { s.press(&["down"]); }
    let l = s.screen(80);
    assert!(l[0].contains("project-18") && l[0].contains("‹") && !l[0].contains("project-00"), "{l:?}");
    s.press(&["enter"]); assert_eq!(s.s.project, "project-18");
}

#[test]
fn a_step_that_has_no_room_on_a_narrow_bar_is_counted_never_cut() {
    let w = coord_world();
    let mut s = open(&w.root, 24);
    s.s.data.host = "runnervm8k3b6-a-long-host-name".into();
    s.s.data.lately = vec!["proj".into()];
    for width in [24, 20] { let l = &s.screen(width)[0]; assert!(l.starts_with(" ○ All ›") && (l.contains("runnervm8k3b6-a-long-host-name") || l.ends_with("+1")) && !l.contains('…'), "at {width}: {l}"); }
}
