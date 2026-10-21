//! With every machine shown, a machine with no task yet has its heading too: folded, `0 tasks`, or open over its
//! coordinators' rows where it has some. Enter or space on its machine coordinator's row talks to it and what you type
//! goes there; the heading comes open with its first task. A filter hides an empty one, and a machine that did not
//! answer has none. The web board has the same heading.
use crate::common;
use common::board::{open, Board};
use common::*;
use serde_json::json;
use sw::board::job::Job;
use sw::board::update::coord_rows::row_id;
use sw::board::update::{update, Effect, Event};
use sw::json::*;

fn has(text: &str, re: &str) -> bool { regex::Regex::new(re).unwrap().is_match(text) }
/// A board of this machine's one task, otherbox with none (and its machine coordinator, that was talked to, where
/// `coords`) and farbox that never answered, every machine shown.
fn empty_otherbox() -> (World, Board) { empty_otherbox_with(true) }
fn empty_otherbox_with(coords: bool) -> (World, Board) {
    let w = world_with(Opts { repo: true, settings: json!({ "board.machines": "otherbox,farbox", "coordinator.agent": "claude" }), ..Default::default() });
    w.add("Mine");
    let mut s = open(&w.root, 40); s.s.side = false;
    update(&mut s.s, Event::Machines(sw::remote::machines()));
    let answer = if coords { json!({ "host": "otherbox", "version": "9.0.0", "coordinator": { "state": "none" }, "coordinators": { "machine": { "state": "idle" } }, "tasks": [] }) }
        else { json!({ "host": "otherbox", "version": "9.0.0", "coordinator": { "off": true }, "coordinators": {}, "tasks": [] }) };
    update(&mut s.s, Event::Remote { label: "otherbox".into(), answer: Ok(answer) });
    for _ in 0..5 { if s.s.scope == "*" { break; } s.press(&["alt+m"]); } assert_eq!(s.s.scope, "*");
    (w, s)
}

#[test]
fn a_machine_with_no_task_has_its_heading_and_its_machine_coordinators_row_under_it_talks_to_it() {
    let (_w, mut s) = empty_otherbox();
    let t = s.screen(100).join("\n");
    assert!(has(&t, r"\n ▾ otherbox ─+\n[^\n]*Main coordinator[^\n]*\n[^\n]*\n[^\n]*Machine coordinator"), "{t}");
    assert!(t.contains("Mine"), "{t}");
    assert!(!has(&t, r"▸ farbox|▾ farbox"), "a machine that did not answer has no heading, its line says why: {t}");
    // Its machine coordinator's row: enter talks to it, what you type goes there.
    s.s.sel = row_id("otherbox", "machine"); s.load();
    let t = s.text();
    assert!(t.contains("enter talk to the otherbox coordinator"), "{t}");
    s.press(&["enter"]);
    s.typed("set up the crm repo");
    let fx = s.hold("enter");
    assert!(fx.iter().any(|e| matches!(e, Effect::Run(Job::Dispatch { machine, text, to, .. }) if machine.label == "otherbox" && text == "set up the crm repo" && to == "machine")), "{fx:?}");
}

#[test]
fn a_machine_with_no_task_and_no_coordinator_has_its_heading_folded_0_tasks_until_its_first_task() {
    let (_w, mut s) = empty_otherbox_with(false);
    let t = s.screen(100).join("\n");
    assert!(has(&t, r"\n ▸ otherbox  0 tasks ─+"), "{t}");
    s.s.sel = sw::board::update::heads::machine_id("otherbox"); s.load();
    s.press(&["enter"]);
    assert!(s.s.msg.contains("No tasks on otherbox yet"), "{}", s.s.msg);
    let now = sw::clock::now();
    update(&mut s.s, Event::Remote { label: "otherbox".into(), answer: Ok(json!({ "host": "otherbox", "version": "9.0.0", "tasks": [{ "id": "crm/1", "title": "Set up crm", "state": "idle", "stage": "todo", "at": now, "created": now }] })) });
    let t = s.screen(100).join("\n");
    assert!(has(&t, r"\n[ ▌]▾ otherbox ─+\n") && t.contains("Set up crm"), "{t}");
}

#[test]
fn this_machine_with_no_task_has_its_heading_too_and_a_filter_hides_the_empty_ones() {
    let w = world_with(Opts { repo: true, settings: json!({ "board.machines": "otherbox" }), ..Default::default() });
    let mut s = open(&w.root, 40); s.s.side = false;
    update(&mut s.s, Event::Machines(sw::remote::machines()));
    let now = sw::clock::now();
    update(&mut s.s, Event::Remote { label: "otherbox".into(), answer: Ok(json!({ "host": "otherbox", "version": "9.0.0", "tasks": [{ "id": "proj/1", "title": "Theirs", "state": "idle", "stage": "todo", "at": now, "created": now }] })) });
    for _ in 0..5 { if s.s.scope == "*" { break; } s.press(&["alt+m"]); } assert_eq!(s.s.scope, "*");
    let here = regex::escape(&sw::shown::words::clean(&sw::remote::check::hostname()));
    let t = s.screen(100).join("\n");
    assert!(has(&t, &format!(r"\n[ ▌]▸ {here}  0 tasks ─+\n")) && has(&t, r"\n[ ▌]▾ otherbox ─+\n"), "{t}");
    s.press(&["ctrl+f"]); s.typed("Theirs");
    let t = s.screen(100).join("\n");
    assert!(!t.contains("0 tasks") && t.contains("Theirs"), "{t}");
}

#[test]
fn the_web_board_shows_an_empty_machine_folded_with_0_tasks() {
    let (_w, s) = empty_otherbox_with(false);
    let groups = sw::webui::model::list::list_of(&s.s, &s.theme).arr("groups").to_vec();
    let g = groups.iter().find(|g| g.s("name") == "otherbox");
    assert!(g.map(|g| g.b("machine") && g.b("folded") && g.s("count") == "0 tasks" && g.arr("rows").is_empty()).unwrap_or(false), "{groups:?}");
}
