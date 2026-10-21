//! With this machine the only one, there is no every machine to show: alt+m asks herdr for the machines first, says
//! when there are none, and shows every machine as soon as there is another; a board that showed every machine goes
//! back to this machine's own when the others are gone. With two machines it goes round as before.
use crate::common;
use common::board::open;
use common::*;
use sw::board::update::{update, Effect, Event};
use sw::remote::Machine;

fn machine(label: &str) -> Machine { Machine { label: label.into(), target: format!("me@{label}"), ..Default::default() } }
fn told(b: &mut common::board::Board, list: Vec<Machine>) { update(&mut b.s, Event::Machines(list)); }

#[test]
fn with_no_other_machine_alt_m_asks_for_them_and_shows_every_machine_only_once_there_is_another() {
    let w = world_with(Opts { repo: true, ..Default::default() });
    w.add("Mine");
    let mut s = open(&w.root, 40);
    // not yet told which machines there are: herdr is asked, and the board stays on this machine's own
    let fx = s.hold("alt+m");
    assert!(fx.iter().any(|e| matches!(e, Effect::Poll(true))), "{fx:?}");
    assert_eq!((s.s.scope.as_str(), s.s.msg.as_str()), ("", "Looking for other machines…"));
    told(&mut s, vec![]);
    assert_eq!(s.s.scope, "");
    assert!(s.s.msg.starts_with("No other machines."), "{}", s.s.msg);
    // herdr has just said: pressed again, it says so at once and asks nothing
    let fx = s.hold("alt+m");
    assert!(!fx.iter().any(|e| matches!(e, Effect::Poll(_))) && s.s.msg.starts_with("No other machines.") && s.s.scope.is_empty(), "{fx:?} {}", s.s.msg);
    // a minute on, herdr is asked again: a machine was added meanwhile, so every machine shows
    sw::clock::advance(61_000);
    s.hold("alt+m");
    assert!(s.s.seeking && s.s.scope.is_empty());
    told(&mut s, vec![machine("otherbox")]);
    assert_eq!(s.s.scope, "*");
    assert!(s.screen(120)[0].starts_with(" ○ All › ") && s.screen(120)[0].contains("otherbox"), "the scope bar offers it: {}", s.text());
    // two machines: round, every machine (All), this machine, then the other one by itself
    s.press(&["alt+m"]); assert_eq!((s.s.scope.as_str(), s.s.level), ("", 1));
    s.press(&["alt+m"]); assert_eq!(s.s.scope, "otherbox");
    s.press(&["alt+m"]); assert_eq!((s.s.scope.as_str(), s.s.level), ("*", 0));
    // the other machine is gone: every machine is this one again, and the board shows this machine's own
    told(&mut s, vec![]);
    assert_eq!((s.s.scope.as_str(), s.s.level), ("", 0));
    assert!(!s.screen(120)[0].contains("otherbox"), "{}", s.text());
}

#[test]
fn a_board_kept_on_every_machine_says_it_is_asking_and_goes_back_to_this_machine_when_there_is_no_other() {
    let w = world_with(Opts { repo: true, ..Default::default() });
    w.add("Mine");
    let mut s = open(&w.root, 60);
    s.s.scope = "*".into();
    assert!(s.screen(120)[0].starts_with(" ○ All › "), "{}", s.text());
    told(&mut s, vec![]);
    assert_eq!((s.s.scope.as_str(), s.s.level), ("", 0));
    assert!(s.screen(120)[0].starts_with(" ○ All › "), "still All, of this machine alone: {}", s.text());
}
