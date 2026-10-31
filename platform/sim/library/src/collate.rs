//! The order the TypeScript reference sorts ids in (`String.prototype.localeCompare`, the root collation), for the
//! characters ids are made of, so that ports, connections and everything sorted by id come out in the same order in
//! both implementations: punctuation first (`_`, `-`, `.`), then digits, then letters with the case ignored; only
//! when two ids are otherwise equal does a lowercase letter come before an uppercase one.

use std::cmp::Ordering;

/// Compares two ids as the reference's `a.localeCompare(b)` does.
pub fn compare(a: &str, b: &str) -> Ordering {
    let primary = |s: &str| s.chars().map(weight).collect::<Vec<_>>();
    primary(a).cmp(&primary(b)).then_with(|| {
        // Tertiary level: at the first difference in case, lowercase first.
        a.chars().zip(b.chars()).map(|(x, y)| x.is_uppercase().cmp(&y.is_uppercase())).find(|o| o.is_ne()).unwrap_or(Ordering::Equal)
    })
}

fn weight(c: char) -> (u8, u32) {
    match c {
        '_' => (1, 0),
        '-' => (1, 1),
        ',' => (1, 2),
        ';' => (1, 3),
        ':' => (1, 4),
        '!' => (1, 5),
        '?' => (1, 6),
        '.' => (1, 7),
        '0'..='9' => (3, c as u32),
        c if c.is_alphabetic() => (4, c.to_lowercase().next().map_or(c as u32, |l| l as u32)),
        // Other symbols: after the punctuation above, before the digits.
        c => (2, c as u32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punctuation_sorts_before_digits_and_digits_before_letters() {
        // The reference: ['roller1.B', 'roller_turntable1.P180'].sort(localeCompare) puts the underscore first.
        assert_eq!(compare("roller_turntable1.P180", "roller1.B"), Ordering::Less);
        assert_eq!(compare("roller1.A", "roller1.B"), Ordering::Less);
        assert_eq!(compare("a.1", "a1"), Ordering::Less);
        assert_eq!(compare("roller10.A", "roller2.A"), Ordering::Less);
    }

    #[test]
    fn case_matters_only_as_a_last_resort() {
        assert_eq!(compare("a", "B"), Ordering::Less);
        assert_eq!(compare("b", "A"), Ordering::Greater);
        assert_eq!(compare("a", "A"), Ordering::Less);
        assert_eq!(compare("abc", "abc"), Ordering::Equal);
    }
}
