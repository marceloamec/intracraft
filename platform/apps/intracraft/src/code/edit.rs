//! The editor's text operations, apart from the window so they are tested on their own: they take the code and the
//! selection (char indices, start and end) and give the new code and the new selection. Indent and outdent, comment
//! and uncomment, a new line that keeps the indentation (one level more after a `:`), insert, find and replace, lines
//! and columns, and the code templates of the Insert menu.

/// Spaces per indentation level (IntraScript counts spaces; PEP 8 says four).
pub const INDENT: &str = "    ";

/// A selection: the char indices of its start and end (equal for a plain cursor).
pub type Selection = (usize, usize);

/// A code template of the Insert menu: its name, its help, its code, and the word selected after inserting it so the
/// user types over it.
pub struct Snippet {
    pub label: &'static str,
    pub help: &'static str,
    pub code: &'static str,
    pub select: Option<&'static str>,
}

/// The templates of the Insert menu, the events first.
pub const SNIPPETS: [Snippet; 14] = [
    Snippet {
        label: "@on_start",
        help: "Runs once when the run starts",
        code: "@on_start\ndef start():\n    pass\n",
        select: Some("pass"),
    },
    Snippet {
        label: "@on_enter",
        help: "Runs when a pallet enters an object",
        code: "@on_enter(OBJECT)\ndef entered(pallet):\n    print(pallet.name, \"entered\", pallet.location.name)\n",
        select: Some("OBJECT"),
    },
    Snippet {
        label: "@on_leave",
        help: "Runs when a pallet leaves an object",
        code: "@on_leave(OBJECT)\ndef left(pallet):\n    pass\n",
        select: Some("OBJECT"),
    },
    Snippet {
        label: "@on_create",
        help: "Runs when a source makes a pallet: give it attributes",
        code: "@on_create(SOURCE)\ndef created(pallet):\n    pallet.destination = \"A\"\n",
        select: Some("SOURCE"),
    },
    Snippet {
        label: "@on_rise",
        help: "Runs when a signal turns True (a photocell sees a pallet)",
        code: "@on_rise(OBJECT.detected)\ndef seen():\n    pass\n",
        select: Some("OBJECT"),
    },
    Snippet {
        label: "@on_fall",
        help: "Runs when a signal turns False",
        code: "@on_fall(OBJECT.detected)\ndef cleared():\n    pass\n",
        select: Some("OBJECT"),
    },
    Snippet {
        label: "@every",
        help: "Runs again and again: a report, a scan",
        code: "@every(1 * h)\ndef report():\n    print(f\"{now()}: \")\n",
        select: Some("1 * h"),
    },
    Snippet { label: "@at", help: "Runs once at a time", code: "@at(2 * h)\ndef later():\n    pass\n", select: Some("2 * h") },
    Snippet {
        label: "@choose_exit",
        help: "Chooses where a turntable sends the pallet",
        code: "@choose_exit(TURNTABLE)\ndef route(pallet):\n    return None  # None: the turntable's own rule\n",
        select: Some("TURNTABLE"),
    },
    Snippet {
        label: "@on_finish",
        help: "Runs when the run ends; it can't wait",
        code: "@on_finish\ndef summary():\n    pass\n",
        select: Some("pass"),
    },
    Snippet {
        label: "hold a conveyor",
        help: "Stop a conveyor, wait, start it again",
        code: "OBJECT.run = False\nwait(2 * s)\nOBJECT.run = True\n",
        select: Some("OBJECT"),
    },
    Snippet {
        label: "wait_until with a timeout",
        help: "Wait for a condition, and say when it took too long",
        code: "if not wait_until(OBJECT.occupied, timeout=30 * s):\n    warn(\"waited 30 s\")\n",
        select: Some("OBJECT"),
    },
    Snippet {
        label: "function",
        help: "A function with its help line",
        code: "def name(value):\n    \"\"\"What it does, in one line.\"\"\"\n    return value\n",
        select: Some("name"),
    },
    Snippet {
        label: "block variable",
        help: "A variable of the block, kept for the whole run",
        code: "counted = 0\n",
        select: Some("counted"),
    },
];

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn ordered(sel: Selection) -> Selection {
    (sel.0.min(sel.1), sel.0.max(sel.1))
}

/// The line (from 1) and the column (from 1) of a char index.
pub fn line_col(text: &str, at: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for c in text.chars().take(at) {
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// The char indices where line `line` (from 1) starts and ends (without its newline); the last line when past the end.
pub fn line_span(text: &str, line: usize) -> Selection {
    let c = chars(text);
    let mut start = 0;
    let mut current = 1;
    for (i, ch) in c.iter().enumerate() {
        if current == line {
            break;
        }
        if *ch == '\n' {
            current += 1;
            start = i + 1;
        }
    }
    let end = c[start.min(c.len())..].iter().position(|ch| *ch == '\n').map_or(c.len(), |p| start + p);
    (start.min(c.len()), end)
}

/// Replaces the selection with `insert`; the cursor goes after it.
pub fn replace(text: &str, sel: Selection, insert: &str) -> (String, Selection) {
    let (a, b) = ordered(sel);
    let c = chars(text);
    let (a, b) = (a.min(c.len()), b.min(c.len()));
    let mut out: String = c[..a].iter().collect();
    out.push_str(insert);
    out.extend(&c[b..]);
    let at = a + insert.chars().count();
    (out, (at, at))
}

/// Inserts a template at the selection, indented as the line it lands on, with its word to type over selected.
pub fn insert_snippet(text: &str, sel: Selection, snippet: &Snippet) -> (String, Selection) {
    let (a, _) = ordered(sel);
    let c = chars(text);
    let line_start = c[..a.min(c.len())].iter().rposition(|ch| *ch == '\n').map_or(0, |p| p + 1);
    let indent: String = c[line_start..].iter().take_while(|ch| **ch == ' ').collect();
    let at_line_start = c[line_start..a.min(c.len())].iter().all(|ch| *ch == ' ');
    let mut body = String::new();
    for (k, line) in snippet.code.lines().enumerate() {
        if k > 0 {
            body.push('\n');
            body.push_str(&indent);
        }
        body.push_str(line);
    }
    if snippet.code.ends_with('\n') {
        body.push('\n');
        if !at_line_start {
            body.push_str(&indent);
        }
    }
    let (out, _) = replace(text, sel, &body);
    let selected = snippet.select.and_then(|w| body.find(w).map(|byte| (body[..byte].chars().count(), w.chars().count())));
    let pick = match selected {
        Some((offset, len)) => (a + offset, a + offset + len),
        None => {
            let end = a + body.chars().count();
            (end, end)
        }
    };
    (out, pick)
}

/// A new line at the selection that keeps the current line's indentation, one level more after a `:`.
pub fn newline(text: &str, sel: Selection) -> (String, Selection) {
    let (a, _) = ordered(sel);
    let c = chars(text);
    let a = a.min(c.len());
    let line_start = c[..a].iter().rposition(|ch| *ch == '\n').map_or(0, |p| p + 1);
    let mut indent: String = c[line_start..a].iter().take_while(|ch| **ch == ' ').collect();
    let before: String = c[line_start..a].iter().collect();
    let code = before.split('#').next().unwrap_or("").trim_end();
    if code.ends_with(':') {
        indent.push_str(INDENT);
    }
    replace(text, sel, &format!("\n{indent}"))
}

/// The lines the selection touches: the char index where each one starts.
fn touched_lines(c: &[char], sel: Selection) -> Vec<usize> {
    let (a, b) = ordered(sel);
    let (a, b) = (a.min(c.len()), b.min(c.len()));
    let first = c[..a].iter().rposition(|ch| *ch == '\n').map_or(0, |p| p + 1);
    // A selection that ends right after a newline doesn't take the line after it.
    let last = if b > a && c[b - 1] == '\n' { b - 1 } else { b };
    let mut starts = vec![first];
    for (i, ch) in c.iter().enumerate().take(last).skip(first) {
        if *ch == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// A position after inserting `n` chars at `at` (a position exactly at `at` moves with the text when `follow`).
fn after_insert(pos: usize, at: usize, n: usize, follow: bool) -> usize {
    if pos > at || (pos == at && follow) { pos + n } else { pos }
}

/// A position after removing `n` chars from `at`.
fn after_remove(pos: usize, at: usize, n: usize) -> usize {
    if pos <= at { pos } else { pos - n.min(pos - at) }
}

/// Moves a selection given in the original text through edits made in order (each at an original index): inserts
/// `(at, n, true)` and removals `(at, n, false)`.
fn shift(sel: Selection, edits: &[(usize, usize, bool)], keep_start: bool) -> Selection {
    let (a, b) = ordered(sel);
    let mut na = a;
    let mut nb = b;
    // The edits are in increasing order of their original index; what they did before a position adds up.
    for &(at, n, insert) in edits.iter().rev() {
        if insert {
            na = after_insert(na, at, n, !keep_start);
            nb = after_insert(nb, at, n, true);
        } else {
            na = after_remove(na, at, n);
            nb = after_remove(nb, at, n);
        }
    }
    (na, nb)
}

/// Indents every line the selection touches by one level, or with `out` takes up to one level of spaces away. A
/// selection that starts at a line's start keeps the whole line selected; a plain cursor moves with its line.
pub fn indent(text: &str, sel: Selection, out: bool) -> (String, Selection) {
    let c = chars(text);
    let starts = touched_lines(&c, sel);
    let mut edits: Vec<(usize, usize, bool)> = Vec::new();
    let mut result = String::with_capacity(text.len() + INDENT.len() * starts.len());
    let mut i = 0;
    for &start in &starts {
        result.extend(&c[i..start]);
        i = start;
        if out {
            let spaces = c[start..].iter().take(INDENT.len()).take_while(|x| **x == ' ').count();
            edits.push((start, spaces, false));
            i = start + spaces;
        } else {
            result.push_str(INDENT);
            edits.push((start, INDENT.len(), true));
        }
    }
    result.extend(&c[i..]);
    (result, shift(sel, &edits, sel.0 != sel.1))
}

/// Comments out every line the selection touches (`# ` at their smallest indentation), or uncomments them when all
/// of them are comments already.
pub fn toggle_comment(text: &str, sel: Selection) -> (String, Selection) {
    let c = chars(text);
    let line_of = |start: usize| -> &[char] {
        let end = c[start..].iter().position(|ch| *ch == '\n').map_or(c.len(), |p| start + p);
        &c[start..end]
    };
    let code_lines: Vec<usize> = touched_lines(&c, sel).into_iter().filter(|&s| line_of(s).iter().any(|ch| *ch != ' ')).collect();
    if code_lines.is_empty() {
        return (text.to_string(), sel);
    }
    let all_commented = code_lines.iter().all(|&s| line_of(s).iter().find(|ch| **ch != ' ') == Some(&'#'));
    let depth = code_lines.iter().map(|&s| line_of(s).iter().take_while(|ch| **ch == ' ').count()).min().unwrap_or(0);
    let mut edits: Vec<(usize, usize, bool)> = Vec::new();
    let mut result = String::with_capacity(text.len() + 2 * code_lines.len());
    let mut i = 0;
    for &start in &code_lines {
        if all_commented {
            let hash = start + line_of(start).iter().position(|ch| *ch == '#').unwrap_or(0);
            let n = if c.get(hash + 1) == Some(&' ') { 2 } else { 1 };
            result.extend(&c[i..hash]);
            edits.push((hash, n, false));
            i = hash + n;
        } else {
            let at = start + depth;
            result.extend(&c[i..at]);
            result.push_str("# ");
            edits.push((at, 2, true));
            i = at;
        }
    }
    result.extend(&c[i..]);
    (result, shift(sel, &edits, sel.0 != sel.1))
}

/// Every place `needle` is found, as char selections, in order; any case unless `case`.
pub fn find_all(text: &str, needle: &str, case: bool) -> Vec<Selection> {
    if needle.is_empty() {
        return Vec::new();
    }
    let fold = |s: &str| -> Vec<char> { if case { s.chars().collect() } else { s.chars().flat_map(char::to_lowercase).collect() } };
    let (mut hay, mut pin) = (fold(text), fold(needle));
    // Lower case changes the length of a few letters: then the search is exact, so the positions stay right.
    if hay.len() != text.chars().count() || pin.len() != needle.chars().count() {
        hay = chars(text);
        pin = chars(needle);
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i + pin.len() <= hay.len() {
        if hay[i..i + pin.len()] == pin[..] {
            out.push((i, i + pin.len()));
            i += pin.len();
        } else {
            i += 1;
        }
    }
    out
}

/// The first match at or after `from`, wrapping round to the first; `back` looks the other way.
pub fn next_match(matches: &[Selection], from: usize, back: bool) -> Option<Selection> {
    if back {
        matches.iter().rev().find(|m| m.1 <= from && m.0 < from).or(matches.last()).copied()
    } else {
        matches.iter().find(|m| m.0 >= from).or(matches.first()).copied()
    }
}

/// Replaces every match of `needle` with `with`: the new code and how many were replaced.
pub fn replace_all(text: &str, needle: &str, with: &str, case: bool) -> (String, usize) {
    let matches = find_all(text, needle, case);
    let c = chars(text);
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    for (a, b) in &matches {
        out.extend(&c[i..*a]);
        out.push_str(with);
        i = *b;
    }
    out.extend(&c[i..]);
    (out, matches.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_and_columns_count_from_one() {
        let t = "ab\ncde\n";
        assert_eq!(line_col(t, 0), (1, 1));
        assert_eq!(line_col(t, 4), (2, 2));
        assert_eq!(line_span(t, 2), (3, 6));
        assert_eq!(line_span(t, 9), (7, 7));
    }

    #[test]
    fn a_new_line_keeps_the_indentation_and_adds_a_level_after_a_colon() {
        let (t, s) = newline("    x = 1", (9, 9));
        assert_eq!((t.as_str(), s), ("    x = 1\n    ", (14, 14)));
        let (t, s) = newline("def f():", (8, 8));
        assert_eq!((t.as_str(), s), ("def f():\n    ", (13, 13)));
        let (t, _) = newline("if x:  # check", (14, 14));
        assert_eq!(t, "if x:  # check\n    ");
    }

    #[test]
    fn indent_and_outdent_the_lines_of_a_selection() {
        let (t, s) = indent("a\nb\nc", (0, 3), false);
        assert_eq!((t.as_str(), s), ("    a\n    b\nc", (0, 11)));
        let (t, s) = indent(&t, s, true);
        assert_eq!((t.as_str(), s), ("a\nb\nc", (0, 3)));
        let (t, s) = indent("x", (1, 1), false);
        assert_eq!((t.as_str(), s), ("    x", (5, 5)));
        let (t, s) = indent("  y", (3, 3), true);
        assert_eq!((t.as_str(), s), ("y", (1, 1)));
    }

    #[test]
    fn comments_toggle_at_the_smallest_indentation() {
        let (t, s) = toggle_comment("if a:\n    b\n", (0, 11));
        assert_eq!(t, "# if a:\n#     b\n");
        let (t, _) = toggle_comment(&t, s);
        assert_eq!(t, "if a:\n    b\n");
        let (t, _) = toggle_comment("    x\n    y", (0, 11));
        assert_eq!(t, "    # x\n    # y");
    }

    #[test]
    fn finds_and_replaces_in_any_case_or_exactly() {
        let t = "Roller1.run = roller1";
        assert_eq!(find_all(t, "roller1", false), [(0, 7), (14, 21)]);
        assert_eq!(find_all(t, "roller1", true), [(14, 21)]);
        let all = find_all(t, "roller1", false);
        assert_eq!(next_match(&all, 8, false), Some((14, 21)));
        assert_eq!(next_match(&all, 15, false), Some((0, 7)));
        assert_eq!(next_match(&all, 14, true), Some((0, 7)));
        assert_eq!(replace_all(t, "roller1", "R2", false), ("R2.run = R2".to_string(), 2));
    }

    #[test]
    fn a_template_lands_indented_with_its_word_selected() {
        let snippet = SNIPPETS.iter().find(|s| s.label == "hold a conveyor").unwrap();
        let (t, s) = insert_snippet("def f():\n    ", (13, 13), snippet);
        assert_eq!(t, "def f():\n    OBJECT.run = False\n    wait(2 * s)\n    OBJECT.run = True\n");
        assert_eq!(t.chars().skip(s.0).take(s.1 - s.0).collect::<String>(), "OBJECT");
    }

    #[test]
    fn every_template_is_valid_intrascript() {
        for s in &SNIPPETS {
            // Templates of code that goes inside a function are checked inside one.
            let code = if s.code.starts_with('@') || s.code.starts_with("def") || s.code.starts_with("counted") {
                s.code.to_string()
            } else {
                format!("@on_start\ndef go():\n{}", s.code.lines().map(|l| format!("    {l}\n")).collect::<String>())
            };
            let problems: Vec<String> = intracraft_script::check(&intracraft_script::ModelInfo::default(), 0, &code, &[])
                .into_iter()
                .filter(|d| d.is_error() && !["unknownName", "unknownMember", "unknownEvent"].contains(&d.code.as_str()))
                .map(|d| d.to_string())
                .collect();
            assert!(problems.is_empty(), "{}: {problems:?}\n{code}", s.label);
        }
    }
}
