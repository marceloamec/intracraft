//! IntraCraft's controller (A08, ADR-0010): If/Then rules over boolean signals. The evaluator is pure: it is given
//! the signals before and now and returns the writes to make, so the 2D run (on events) and the 3D run (every 10 ms)
//! share exactly the same logic. Delays are the caller's to schedule. Signal names are `<instance id>.<signal>`.

use std::collections::BTreeMap;

use intracraft_library::layout::{Condition, Edge, Rule};

/// Signal values by name; a missing signal is false.
pub type Signals = BTreeMap<String, bool>;

/// A write a rule asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct SignalWrite {
    pub signal: String,
    pub value: bool,
    /// After how long it takes effect, ms.
    pub delay_ms: f64,
    pub rule_id: String,
}

/// The writes of every rule whose conditions all hold, in the rules' order.
pub fn evaluate_rules(rules: &[Rule], previous: &Signals, current: &Signals) -> Vec<SignalWrite> {
    rules
        .iter()
        .filter(|r| r.when.iter().all(|c| holds(c, previous, current)))
        .flat_map(|r| {
            r.then.iter().map(|a| SignalWrite {
                signal: a.signal.clone(),
                value: a.value,
                delay_ms: a.delay_ms.unwrap_or(0.0),
                rule_id: r.id.clone(),
            })
        })
        .collect()
}

fn holds(c: &Condition, previous: &Signals, current: &Signals) -> bool {
    let get = |s: &Signals, name: &str| s.get(name).copied().unwrap_or(false);
    match c {
        Condition::Is { signal, is } => get(current, signal) == *is,
        Condition::Edge { signal, edge } => {
            let (before, now) = (get(previous, signal), get(current, signal));
            match edge {
                Edge::Rising => !before && now,
                Edge::Falling => before && !now,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intracraft_library::layout::Action;

    fn signals(v: &[(&str, bool)]) -> Signals {
        v.iter().map(|(k, b)| (k.to_string(), *b)).collect()
    }

    fn rule(id: &str, when: Vec<Condition>, then: &[(&str, bool, Option<f64>)]) -> Rule {
        Rule { id: id.into(), when, then: then.iter().map(|(s, v, d)| Action { signal: s.to_string(), value: *v, delay_ms: *d }).collect() }
    }

    #[test]
    fn stops_a_conveyor_when_its_photocell_is_blocked() {
        let stop = rule("stop", vec![Condition::Is { signal: "pe1.blocked".into(), is: true }], &[("roller1.run", false, None)]);
        assert!(evaluate_rules(std::slice::from_ref(&stop), &Signals::new(), &Signals::new()).is_empty());
        let w = evaluate_rules(&[stop], &Signals::new(), &signals(&[("pe1.blocked", true)]));
        assert_eq!(w, [SignalWrite { signal: "roller1.run".into(), value: false, delay_ms: 0.0, rule_id: "stop".into() }]);
    }

    #[test]
    fn edges_fire_once_on_the_change() {
        let r = rule("r", vec![Condition::Edge { signal: "s".into(), edge: Edge::Rising }], &[("out", true, Some(10_000.0))]);
        let (off, on) = (signals(&[("s", false)]), signals(&[("s", true)]));
        assert_eq!(evaluate_rules(std::slice::from_ref(&r), &off, &on).len(), 1);
        assert!(evaluate_rules(std::slice::from_ref(&r), &on, &on).is_empty());
        assert_eq!(evaluate_rules(std::slice::from_ref(&r), &off, &on)[0].delay_ms, 10_000.0);
        let f = rule("f", vec![Condition::Edge { signal: "s".into(), edge: Edge::Falling }], &[("out", false, None)]);
        assert_eq!(evaluate_rules(std::slice::from_ref(&f), &on, &off).len(), 1);
        assert!(evaluate_rules(&[f], &off, &on).is_empty());
    }

    #[test]
    fn every_condition_must_hold() {
        let r = rule(
            "and",
            vec![Condition::Is { signal: "a".into(), is: true }, Condition::Is { signal: "b".into(), is: false }],
            &[("x", true, None), ("y", true, None)],
        );
        assert_eq!(evaluate_rules(std::slice::from_ref(&r), &Signals::new(), &signals(&[("a", true)])).len(), 2);
        assert!(evaluate_rules(&[r], &Signals::new(), &signals(&[("a", true), ("b", true)])).is_empty());
    }
}
