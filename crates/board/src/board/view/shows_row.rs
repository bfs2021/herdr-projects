//! The filter row under the header: `All 48 · Needs you 3 · Working 2 · Ready 1 · To do 12 · Done 30 · Coordinators 4`,
//! the one shown lit, the others greyed with their counts (a count of tasks that need you in red). Where it has no
//! room, the names are said shorter (then shortest), those with nothing in them go, and where that is still too long only the one
//! shown stands, with the key that moves on. Where each is drawn is kept for a click (update/mouse.rs).
use crate::board::keys::key_name;
use crate::board::state::State;
use crate::board::theme::Theme;
use crate::board::update::shows::SHOWS;
use crate::shown::ink::{Ink, Inked};

const SHORT: [&str; 7] = ["All", "Needs", "Working", "Ready", "To do", "Done", "Coords"];
const TINY: [&str; 7] = ["All", "Need", "Work", "Rdy", "Todo", "Done", "Co"];
/// The row, `w` cells wide, and where each filter landed: (first cell, last cell, which).
pub fn shows_line(s: &State, t: &Theme, w: usize) -> (Inked, Vec<(usize, usize, usize)>) {
    // `only`: 0 every filter, 1 those with something in them (and the one shown), 2 the one shown alone.
    let lay = |names: &[&str], only: u8| {
        let (mut line, mut hits) = (Inked::plain(" "), vec![]);
        for (n, name) in names.iter().enumerate() {
            let count = s.show_n.get(n).copied().unwrap_or(0);
            if (only == 2 && n != s.show) || (only == 1 && n != s.show && n != 0 && count == 0) { continue; }
            if line.width() > 1 { line = line.add(Inked::of(" · ", Ink::Muted)); }
            let lit = n == s.show;
            let named = if lit { Inked::of(*name, Ink::Accent).bold() } else { Inked::of(*name, Ink::Muted) };
            let num = Inked::of(format!(" {count}"), if n == 1 && count > 0 { Ink::Red } else if lit { Ink::Accent } else { Ink::Muted });
            let piece = named.add(num);
            let piece = if lit { if t.paints_selection() { piece.on(Ink::Selection) } else { piece.inv() } } else { piece };
            let from = line.width();
            line = line.add(piece);
            hits.push((from, line.width(), n));
        }
        if only == 2 { line = line.add(Inked::of(format!("  {} next", key_name("filter")), Ink::Muted)); }
        (line, hits)
    };
    let names: Vec<&str> = SHOWS.iter().map(|x| x.0).collect();
    for (list, only) in [(&names[..], 0), (&SHORT[..], 0), (&SHORT[..], 1), (&TINY[..], 0), (&TINY[..], 1), (&names[..], 2), (&SHORT[..], 2)] {
        let (line, hits) = lay(list, only);
        if line.width() <= w || (only == 2 && list == &SHORT[..]) { return (line.cut(w), hits.into_iter().filter(|h| h.1 <= w).collect()); }
    }
    unreachable!()
}
