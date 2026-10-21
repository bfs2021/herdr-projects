//! What a row has: its mark, its title as it reads, what it says, and what it has for each column on the right. A task's
//! come from shown/ (the same words herdr's sidebar and the web page say); an automation's row (a record `{id, auto}`)
//! has its own: the word on or off where a task has its number, its trigger where a task says something, the task its
//! last firing made where the review tag stands, and when it fires next or fired last in the time column.
use crate::board::state::State;
use crate::board::update::pending::said as pending_said;
use crate::board::theme::Theme;
use crate::json::*;
use crate::research::is_research;
use crate::shown::ink::{Ink, Inked};
use crate::shown::queue::queued;
use crate::shown::row_says::{row_fields, sentence};
use crate::shown::states::{board_step, in_review, is_live, reviewer_mark, status_ink};

/// The automations' mark: an arrow going round, where every task's mark is round, a diamond or a sign.
pub const AUTO_MARK: &str = "⟳";
#[derive(Clone, Debug, Default)]
pub struct F { pub bar: Inked, pub percent: Inked, pub eta: Inked, pub review: Inked, pub pr: Inked, pub tag: Inked, pub lead: Inked, pub elapsed: String, pub ago: bool, pub never: bool, pub when: String }
impl F {
    pub fn has(&self, key: &str) -> bool {
        match key { "bar" => !self.bar.is_empty(), "percent" => !self.percent.is_empty(), "eta" => !self.eta.is_empty(), "review" => !self.review.is_empty(), "pr" => !self.pr.is_empty(), "tag" => !self.tag.is_empty(), "lead" => !self.lead.is_empty(), "time" => true, _ => false }
    }
    pub fn get(&self, key: &str) -> Inked {
        match key { "bar" => self.bar.clone(), "percent" => self.percent.clone(), "eta" => self.eta.clone(), "review" => self.review.clone(), "pr" => self.pr.clone(), "tag" => self.tag.clone(), "lead" => self.lead.clone(), _ => Inked::new() }
    }
}
pub fn is_auto(r: &Obj) -> bool { r.has("auto") }
/// A coordinator's row (update/coord_rows.rs: rec): drawn as a task agent's, with only what a coordinator has.
pub fn is_coord(r: &Obj) -> bool { r.has("coord") }
/// The rows whose columns are their own (an automation's, a coordinator's): a tag, a lead and the time.
pub fn own_cols(r: &Obj) -> bool { is_auto(r) || is_coord(r) }
/// A row of two lines: a live task's, a coordinator's.
pub fn two_lines(r: &Obj) -> bool { is_live(r) || is_coord(r) }
/// A coordinator's mark: the chat's for where it stands, autopilot's `◎` while that is on.
fn coord_mark(t: &Theme, c: &Value) -> Inked {
    let ink = match c.s("kind") { "working" => Ink::Yellow, "blocked" | "error" => Ink::Red, "idle" => Ink::Green, "pilot" => match c.s("pilot") { "needs you" | "could not start" => Ink::Red, "working" => Ink::Yellow, _ => Ink::Plain }, _ => Ink::Muted };
    // Autopilot's mark in the colour of its state, as its heading had it: bold where it needs you, quiet asleep.
    if c.s("kind") == "pilot" { let m = Inked::of(super::pilot::PILOT_MARK, ink); return if ink == Ink::Red { m.bold() } else if c.s("pilot") == "asleep" { m.quiet() } else { m }; }
    let m = Inked::of(t.mark(c.s("kind"), ""), ink);
    if ink == Ink::Red { m.bold() } else { m }
}
pub fn auto(r: &Obj) -> &Value { r.at("auto") }
/// A task's mark: herdr's own, a different one for each state, in the colour of its group.
pub fn mark(t: &Theme, r: &Obj) -> Inked {
    if is_auto(r) { let on = auto(r).b("on"); let m = Inked::of(AUTO_MARK, Ink::Auto); return if on { m } else { m.quiet() }; }
    if is_coord(r) { return coord_mark(t, r.at("coord")); }
    // Something you just did is under way on it (update/pending.rs): the working mark, or the error mark when it failed.
    let p = r.at("pending");
    if p.b("word") { return if p.b("failed") { Inked::of(t.mark("error", ""), Ink::Red).bold() } else { Inked::of(t.mark("working", ""), Ink::Yellow) }; }
    let c = status_ink(r);
    if r.b("archivedAt") { return Inked::of(t.mark("unknown", ""), c); }
    if in_review(r) { return Inked::of(t.mark("ready", ""), c).bold(); } // passed and waits for your yes
    if reviewer_mark(r) { return Inked::of(t.mark("review", ""), c); } // the reviewer at work
    if board_step(r) { return Inked::of(t.mark("board", ""), c); } // a step of the board's own
    if queued(r) { return Inked::of(t.mark("queue", ""), c); } // it waits for a free slot
    if r.s("state") == "blocked" { return Inked::of(t.mark("blocked", r.s("kind")), c).bold(); } // what it needs from you
    if r.s("state") == "done" { return Inked::of(t.mark("done", ""), c).quiet(); } // finished: the same mark, quieter
    let st = r.s("state");
    Inked::of(t.mark(if ["working", "error", "done"].contains(&st) { st } else { "idle" }, ""), c)
}
/// The title as the row reads it, with what is said of it in brackets.
pub fn row_title(s: &State, r: &Obj) -> String {
    if is_auto(r) { return crate::shown::words::clean(auto(r).s("name")); }
    if is_coord(r) { let sum = r.at("coord").t("sum"); return crate::shown::words::clean(&if sum.is_empty() { r.t("title") } else { format!("{} · {sum}", r.s("title")) }); }
    let mut t = crate::shown::words::clean(if r.b("title") { r.s("title") } else { r.s("id") });
    if r.arr("attempts").len() > 1 { t.push_str(&format!(" ×{}", r.arr("attempts").len())); }
    if crate::chat_only::is_chat(r) { t.push_str(" [chat]"); }
    if is_research(r) { t.push_str(" [research]"); }
    if r.at("automation").b("id") { t.push_str(" [auto]"); }
    if r.b("archivedAt") && !s.archive_view { t.push_str(" [archived]"); }
    t
}
/// The title is plain whatever the task's state: the mark and the heading carry the state's colour. A live task's is
/// bold. An automation's name is plain, greyed while it is off; the `[auto]` tag is in the automations' colour.
pub fn name_ink(r: &Obj, s: &str) -> Inked {
    if is_auto(r) { return if auto(r).b("on") { Inked::plain(s) } else { Inked::of(s, Ink::Muted) }; }
    if is_coord(r) {
        // The main Coordinator's row sums up the others after its name: quiet, what needs you in red.
        let (name, sum) = s.split_once(" · ").map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((s.to_string(), String::new()));
        let m = Inked::plain(name);
        let m = if ["working", "blocked", "error"].contains(&r.at("coord").s("kind")) { m.bold() } else { m };
        if sum.is_empty() { return m; }
        let (on, need) = sum.split_once(" · ").map(|(a, b)| (a.to_string(), format!(" · {b}"))).unwrap_or((sum, String::new()));
        return m.add(Inked::of(format!(" · {on}"), Ink::Muted)).add(Inked::of(need, Ink::Red));
    }
    let bold = ["working", "blocked", "error"].contains(&r.s("state"));
    let mut out = Inked::new();
    if r.at("automation").b("id") {
        let re = crate::re!(r"\[auto\]( \[archived\])?$");
        if let Some(m) = re.find(s) {
            out.push(&s[..m.start()], Ink::Plain);
            let tag = if r.s("state") == "done" { Inked::of("[auto]", Ink::Auto).quiet() } else { Inked::of("[auto]", Ink::Auto) };
            out = out.add(tag);
            out.push(&s[m.start() + 6..], Ink::Plain);
            return if bold { out.bold() } else { out };
        }
    }
    out.push(s, Ink::Plain);
    if bold { out.bold() } else { out }
}
/// What a row says on its first line: nothing on a live task's (it says it on its second). An automation says its
/// trigger there, on a narrow screen (`tall`) under its name instead.
pub fn says_beside(s: &State, r: &Obj, tall: bool) -> Inked {
    if is_auto(r) { return if tall { Inked::new() } else { auto_said(s, r, "beside") }; }
    if is_coord(r) { return Inked::new(); }
    if is_live(r) { Inked::new() } else { pending_said(r.at("pending")).unwrap_or_else(|| sentence(r)) }
}
/// What a live task's second line says: its words, or something true and quiet instead of a gap.
pub fn says_under(s: &State, r: &Obj, says_beside_shown: bool) -> Inked {
    if is_auto(r) { return auto_said(s, r, if says_beside_shown { "why" } else { "all" }); }
    if is_coord(r) {
        let c = r.at("coord");
        let red = c.s("word") == "needs you" || ["needs you", "could not start"].contains(&c.s("pilot"));
        let said = c.t("said");
        // On autopilot its first word is its state's, in red where it needs you; the rest in the grey a task's line has.
        let (word, rest) = if c.b("pilot") { said.split_once(" · ").map(|(a, b)| (a.to_string(), format!(" · {b}"))).unwrap_or((said.clone(), String::new())) } else { (said.clone(), String::new()) };
        return Inked::of(word, if red { Ink::Red } else { Ink::Soft }).add(Inked::of(rest, Ink::Soft));
    }
    if let Some(p) = pending_said(r.at("pending")) { return p; }
    let said = sentence(r);
    if !said.is_empty() { return said; }
    let st = r.s("stage");
    Inked::of(if st == "progress" { "Yours: no agent on it" } else if in_review(r) { "Not reviewed" } else if reviewer_mark(r) { "Under review" } else if r.s("state") == "working" { "Working" } else { "Needs you" }, Ink::Soft)
}
/// What an automation's row says: its trigger, its repo where that is not the board's (never under its repo's own
/// heading); and why the board turned it off. Its machine is the heading it stands under. `part`: "beside" its name,
/// "why" (under it), or "all" under it.
pub fn auto_said(s: &State, r: &Obj, part: &str) -> Inked {
    let a = auto(r);
    if part != "why" { if let Some(p) = pending_said(a.at("pending")) { return p; } }
    let row = a.at("rowNow");
    let first = [if s.tags_gone { row.t("state") } else { String::new() }, row.t("trigger"), if row.s("repo") != s.data.repo && !s.by_repo { crate::shown::words::repo_label(row.s("repo")) } else { String::new() }];
    let mut parts: Vec<String> = if part == "why" { vec![] } else { first.to_vec() };
    if part != "beside" { parts.push(row.t("why")); }
    let said = parts.into_iter().filter(|x| !x.is_empty()).map(|x| crate::shown::words::clean(&x)).collect::<Vec<_>>().join(" · ");
    if said.is_empty() { Inked::new() } else if a.b("on") { Inked::of(said, Ink::Soft) } else { Inked::of(said, Ink::Muted) }
}
pub fn fields(s: &State, t: &Theme, r: &Obj, now: i64) -> F {
    if is_auto(r) {
        let a = auto(r);
        let row = a.at("rowNow");
        let ink = |x: String| Inked::of(x, Ink::Muted);
        let made = if row.b("made") { let id = if a.b("host") { format!("{}:{}", a.s("host"), row.s("made")) } else { row.t("made") }; s.items.iter().find(|x| x.s("id") == id).cloned() } else { None };
        let tag = made.map(|m| mark(t, &m).add(Inked::plain(" ")).add(Inked::of(crate::board::tags::number_tag(&m, &s.data.keys, &s.scope, false), Ink::Muted))).unwrap_or_default();
        return F { tag, lead: ink(row.t("lead")), elapsed: row.t("age"), ago: row.b("ago"), never: !row.b("age"), when: row.t("when"), ..Default::default() };
    }
    if is_coord(r) {
        // Its time: since it last said something; on autopilot what autopilot shows where a task has its columns.
        let c = r.at("coord");
        let at = c.i("at");
        // The last task autopilot added: with its mark and its tag where the board has it, as an automation's last task.
        let item = if c.b("lastId") { let at = if r.b("host") { format!("{}:{}", r.s("host"), c.s("lastId")) } else { c.t("lastId") }; s.items.iter().find(|x| x.s("id") == at) } else { None };
        let tag = match item { Some(m) => mark(t, m).add(Inked::plain(" ")).add(Inked::of(crate::board::tags::number_tag(m, &s.data.keys, &s.scope, false), Ink::Muted)), None => Inked::of(c.t("last"), Ink::Muted) };
        return F { tag: Inked::of(c.t("added"), Ink::Muted), lead: tag, elapsed: if c.b("next") { c.t("next") } else if at > 0 { crate::shown::words::age(now - at) } else { String::new() }, ago: at > 0 && !c.b("next"), ..Default::default() };
    }
    let f = row_fields(r, now);
    F { bar: f.bar, percent: f.percent, eta: f.eta, review: f.review, pr: f.pr, elapsed: f.elapsed, ago: f.ago, ..Default::default() }
}
