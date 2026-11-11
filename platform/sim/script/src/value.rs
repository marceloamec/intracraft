//! The values IntraScript code works with: Python's (None, booleans, integers, floats, text, lists, tuples, dictionaries)
//! plus physical quantities, references to objects of the layout and to pallets, and functions.
//!
//! Lists and dictionaries are shared (`Arc<Mutex<…>>`), as in Python. A lock is only ever held for one primitive
//! operation on the container and never while other code runs, so a list that contains itself can't deadlock: code that
//! must look inside (equality, display) copies the items out first and stops at a fixed depth.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::units::{Dim, display_quantity};

/// A list of values, shared.
pub type ListRef = Arc<Mutex<Vec<Value>>>;
/// A dictionary, shared.
pub type DictRef = Arc<Mutex<Dict>>;

/// How deep equality and display look into containers before they give up (a container inside itself).
const MAX_DEPTH: usize = 40;

#[derive(Clone, Debug, Default)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    /// A number with a physical dimension, in SI units; never dimensionless (that is a `Float`).
    Qty(f64, Dim),
    Str(Arc<str>),
    List(ListRef),
    Tuple(Arc<Vec<Value>>),
    Dict(DictRef),
    Range(i64, i64, i64),
    /// An object of the layout: its index in the model description.
    Object(u32),
    /// A pallet of the run: its id.
    Pallet(u64),
    /// A function of a code block.
    Func {
        block: u32,
        func: u32,
    },
    Builtin(crate::builtins::Builtin),
    /// A method looked up on a value, waiting for its call: `pallets.append`.
    Method(Arc<(Value, Arc<str>)>),
    Iter(Arc<Mutex<Iter>>),
    /// A slice written in brackets, `a[1:3]`.
    Slice(Option<i64>, Option<i64>, Option<i64>),
    /// A local variable that has no value yet.
    Unbound,
}

/// Where an iteration stands.
#[derive(Debug)]
pub enum Iter {
    /// A list is read live, as Python does: items appended while looping are visited too.
    List(ListRef, usize),
    Items(Vec<Value>, usize),
    Range(i64, i64, i64),
    Enumerate(Box<Iter>, i64),
    Zip(Vec<Iter>),
}

impl Iterator for Iter {
    type Item = Value;

    fn next(&mut self) -> Option<Value> {
        match self {
            Iter::List(list, i) => {
                let v = lock(list).get(*i).cloned();
                *i += 1;
                v
            }
            Iter::Items(items, i) => {
                let v = items.get(*i).cloned();
                *i += 1;
                v
            }
            Iter::Range(cur, stop, step) => {
                let more = if *step > 0 { *cur < *stop } else { *cur > *stop };
                if !more {
                    return None;
                }
                let v = *cur;
                *cur = cur.saturating_add(*step);
                Some(Value::Int(v))
            }
            Iter::Enumerate(inner, n) => {
                let v = inner.next()?;
                let k = *n;
                *n += 1;
                Some(Value::Tuple(Arc::new(vec![Value::Int(k), v])))
            }
            Iter::Zip(inners) => {
                let mut items = Vec::with_capacity(inners.len());
                for i in inners.iter_mut() {
                    items.push(i.next()?);
                }
                Some(Value::Tuple(Arc::new(items)))
            }
        }
    }
}

/// Locks a container; a lock poisoned by a panic elsewhere is still usable (the data is plain values).
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Value {
    pub fn str(s: &str) -> Value {
        Value::Str(Arc::from(s))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Arc::new(Mutex::new(items)))
    }

    pub fn tuple(items: Vec<Value>) -> Value {
        Value::Tuple(Arc::new(items))
    }

    pub fn dict(d: Dict) -> Value {
        Value::Dict(Arc::new(Mutex::new(d)))
    }

    /// A number with a dimension; dimensionless gives a float.
    pub fn qty(x: f64, dim: Dim) -> Value {
        if dim.is_none() { Value::Float(x) } else { Value::Qty(x, dim) }
    }

    /// The type's name as error messages show it.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "None",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Qty(_, d) => d.name(),
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Tuple(_) => "tuple",
            Value::Dict(_) => "dict",
            Value::Range(..) => "range",
            Value::Object(_) => "object",
            Value::Pallet(_) => "pallet",
            Value::Func { .. } | Value::Builtin(_) | Value::Method(_) => "function",
            Value::Iter(_) => "iterator",
            Value::Slice(..) => "slice",
            Value::Unbound => "unbound",
        }
    }

    /// Python's truth: zero, empty and None are false.
    pub fn truthy(&self) -> bool {
        match self {
            Value::None | Value::Unbound => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(x) | Value::Qty(x, _) => *x != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !lock(l).is_empty(),
            Value::Tuple(t) => !t.is_empty(),
            Value::Dict(d) => !lock(d).is_empty(),
            Value::Range(a, b, step) => {
                if *step > 0 {
                    a < b
                } else {
                    a > b
                }
            }
            _ => true,
        }
    }

    /// A number as a float, if it is a plain number (bool, int or float).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Bool(b) => Some(f64::from(u8::from(*b))),
            Value::Int(i) => Some(*i as f64),
            Value::Float(x) => Some(*x),
            _ => None,
        }
    }

    /// The value and dimension of any number (plain numbers are dimensionless).
    pub fn as_quantity(&self) -> Option<(f64, Dim)> {
        match self {
            Value::Qty(x, d) => Some((*x, *d)),
            v => v.as_f64().map(|x| (x, Dim::NONE)),
        }
    }

    /// Python's `==`.
    pub fn equals(&self, other: &Value) -> bool {
        eq_depth(self, other, 0)
    }

    /// What `str()` and `print` show.
    pub fn to_text(&self) -> String {
        match self {
            Value::Str(s) => s.to_string(),
            v => v.repr(),
        }
    }

    /// What a value looks like inside a list, or `repr()`.
    pub fn repr(&self) -> String {
        let mut out = String::new();
        repr_into(self, &mut out, 0);
        out
    }
}

fn eq_depth(a: &Value, b: &Value, depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match (a, b) {
        (Value::None, Value::None) => true,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Qty(x, d), Value::Qty(y, e)) => d == e && x == y,
        (Value::Object(x), Value::Object(y)) => x == y,
        (Value::Pallet(x), Value::Pallet(y)) => x == y,
        (Value::Func { block: a, func: f }, Value::Func { block: b, func: g }) => a == b && f == g,
        (Value::Builtin(x), Value::Builtin(y)) => x == y,
        (Value::Range(a, b, c), Value::Range(d, e, f)) => (a, b, c) == (d, e, f),
        (Value::List(x), Value::List(y)) => {
            if Arc::ptr_eq(x, y) {
                return true;
            }
            let (xs, ys) = (lock(x).clone(), lock(y).clone());
            xs.len() == ys.len() && xs.iter().zip(&ys).all(|(p, q)| eq_depth(p, q, depth + 1))
        }
        (Value::Tuple(x), Value::Tuple(y)) => x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| eq_depth(p, q, depth + 1)),
        (Value::Dict(x), Value::Dict(y)) => {
            if Arc::ptr_eq(x, y) {
                return true;
            }
            let (xs, ys) = (lock(x).items(), lock(y).items());
            xs.len() == ys.len()
                && xs.iter().all(|(k, v)| ys.iter().any(|(k2, v2)| eq_depth(k, k2, depth + 1) && eq_depth(v, v2, depth + 1)))
        }
        _ => match (a.as_f64(), b.as_f64()) {
            (Some(x), Some(y)) => match (a, b) {
                (Value::Int(i), Value::Int(j)) => i == j,
                _ => x == y,
            },
            _ => false,
        },
    }
}

fn repr_into(v: &Value, out: &mut String, depth: usize) {
    if depth > MAX_DEPTH {
        out.push_str("...");
        return;
    }
    match v {
        Value::None => out.push_str("None"),
        Value::Bool(true) => out.push_str("True"),
        Value::Bool(false) => out.push_str("False"),
        Value::Int(i) => out.push_str(&i.to_string()),
        Value::Float(x) => out.push_str(&float_text(*x)),
        Value::Qty(x, d) => out.push_str(&display_quantity(*x, *d)),
        Value::Str(s) => {
            out.push('\'');
            for c in s.chars() {
                match c {
                    '\'' => out.push_str("\\'"),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    c => out.push(c),
                }
            }
            out.push('\'');
        }
        Value::List(l) => {
            let items = lock(l).clone();
            seq_into(&items, '[', ']', out, depth);
        }
        Value::Tuple(t) => {
            if t.len() == 1 {
                out.push('(');
                repr_into(&t[0], out, depth + 1);
                out.push_str(",)");
            } else {
                seq_into(t, '(', ')', out, depth);
            }
        }
        Value::Dict(d) => {
            let items = lock(d).items();
            out.push('{');
            for (i, (k, v)) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                repr_into(k, out, depth + 1);
                out.push_str(": ");
                repr_into(v, out, depth + 1);
            }
            out.push('}');
        }
        Value::Range(a, b, s) => {
            if *s == 1 {
                out.push_str(&format!("range({a}, {b})"));
            } else {
                out.push_str(&format!("range({a}, {b}, {s})"));
            }
        }
        Value::Object(i) => out.push_str(&format!("<object {i}>")),
        Value::Pallet(id) => out.push_str(&format!("<pallet {id}>")),
        Value::Func { .. } | Value::Builtin(_) | Value::Method(_) => out.push_str("<function>"),
        Value::Iter(_) => out.push_str("<iterator>"),
        Value::Slice(..) => out.push_str("<slice>"),
        Value::Unbound => out.push_str("<no value>"),
    }
}

fn seq_into(items: &[Value], open: char, close: char, out: &mut String, depth: usize) {
    out.push(open);
    for (i, v) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        repr_into(v, out, depth + 1);
    }
    out.push(close);
}

/// A float as Python prints it: `1.0`, `0.25`, `1e+20`, `inf`, `nan`.
pub fn float_text(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let s = format!("{x:?}");
    match s.split_once('e') {
        Some((m, e)) if !e.starts_with('-') => format!("{m}e+{e}"),
        _ => s,
    }
}

/// A dictionary key: the values Python can hash, in a total order so lookups are deterministic.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    None,
    Int(i64),
    /// A float that is not a whole number: its bits, ordered.
    Float(u64),
    Str(Arc<str>),
    Tuple(Vec<Key>),
    Object(u32),
    Pallet(u64),
}

impl Key {
    /// The key of a value, or why it can't be one.
    pub fn of(v: &Value) -> Result<Key, String> {
        Ok(match v {
            Value::None => Key::None,
            Value::Bool(b) => Key::Int(i64::from(*b)),
            Value::Int(i) => Key::Int(*i),
            Value::Float(x) => {
                if x.is_nan() {
                    return Err("nan can't be a dictionary key".into());
                }
                if x.fract() == 0.0 && x.abs() < 9.0e15 {
                    Key::Int(*x as i64)
                } else {
                    let bits = x.to_bits();
                    // Flip so the unsigned order is the numeric order.
                    Key::Float(if bits >> 63 == 1 { !bits } else { bits | (1 << 63) })
                }
            }
            Value::Str(s) => Key::Str(s.clone()),
            Value::Tuple(t) => Key::Tuple(t.iter().map(Key::of).collect::<Result<_, _>>()?),
            Value::Object(i) => Key::Object(*i),
            Value::Pallet(p) => Key::Pallet(*p),
            other => return Err(format!("a {} can't be a dictionary key", other.type_name())),
        })
    }
}

/// A dictionary that keeps the order its keys were added in, as Python's does.
#[derive(Clone, Debug, Default)]
pub struct Dict {
    entries: Vec<Option<(Value, Value)>>,
    index: BTreeMap<Key, usize>,
}

impl Dict {
    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn get(&self, key: &Value) -> Result<Option<Value>, String> {
        let k = Key::of(key)?;
        Ok(self.index.get(&k).and_then(|&i| self.entries.get(i)).and_then(|e| e.as_ref()).map(|(_, v)| v.clone()))
    }

    pub fn contains(&self, key: &Value) -> Result<bool, String> {
        Ok(self.index.contains_key(&Key::of(key)?))
    }

    pub fn set(&mut self, key: Value, value: Value) -> Result<(), String> {
        let k = Key::of(&key)?;
        match self.index.get(&k) {
            Some(&i) => {
                if let Some(slot) = self.entries.get_mut(i) {
                    *slot = Some((key, value));
                }
            }
            None => {
                self.index.insert(k, self.entries.len());
                self.entries.push(Some((key, value)));
            }
        }
        Ok(())
    }

    pub fn remove(&mut self, key: &Value) -> Result<Option<Value>, String> {
        let k = Key::of(key)?;
        let Some(i) = self.index.remove(&k) else { return Ok(None) };
        let old = self.entries.get_mut(i).and_then(Option::take).map(|(_, v)| v);
        if self.entries.len() > 32 && self.entries.len() > 2 * self.index.len() {
            self.compact();
        }
        Ok(old)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
    }

    /// Keys and values, in order.
    pub fn items(&self) -> Vec<(Value, Value)> {
        self.entries.iter().flatten().cloned().collect()
    }

    pub fn keys(&self) -> Vec<Value> {
        self.entries.iter().flatten().map(|(k, _)| k.clone()).collect()
    }

    fn compact(&mut self) {
        let items = self.items();
        self.clear();
        for (k, v) in items {
            // The keys were valid already.
            let _ = self.set(k, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_values_as_python_does() {
        assert_eq!(Value::Float(1.0).repr(), "1.0");
        assert_eq!(Value::Float(0.25).repr(), "0.25");
        assert_eq!(Value::Float(1e20).repr(), "1e+20");
        assert_eq!(Value::list(vec![Value::Int(1), Value::str("a"), Value::None]).repr(), "[1, 'a', None]");
        assert_eq!(Value::tuple(vec![Value::Int(1)]).repr(), "(1,)");
        assert_eq!(Value::str("hi").to_text(), "hi");
    }

    #[test]
    fn a_list_inside_itself_neither_hangs_nor_crashes() {
        let l = Value::list(vec![Value::Int(1)]);
        if let Value::List(inner) = &l {
            lock(inner).push(l.clone());
        }
        assert!(l.repr().contains("..."));
        assert!(l.equals(&l));
    }

    #[test]
    fn whole_numbers_are_one_key_whatever_their_type() {
        let mut d = Dict::default();
        d.set(Value::Int(1), Value::str("a")).unwrap();
        d.set(Value::Float(1.0), Value::str("b")).unwrap();
        d.set(Value::Bool(true), Value::str("c")).unwrap();
        assert_eq!(d.len(), 1);
        assert_eq!(d.get(&Value::Int(1)).unwrap().unwrap().to_text(), "c");
        assert!(d.set(Value::list(vec![]), Value::None).is_err());
    }

    #[test]
    fn a_dictionary_keeps_the_order_its_keys_came_in() {
        let mut d = Dict::default();
        for k in ["b", "a", "c"] {
            d.set(Value::str(k), Value::None).unwrap();
        }
        d.remove(&Value::str("a")).unwrap();
        d.set(Value::str("a"), Value::None).unwrap();
        let keys: Vec<String> = d.keys().iter().map(Value::to_text).collect();
        assert_eq!(keys, ["b", "c", "a"]);
    }
}
