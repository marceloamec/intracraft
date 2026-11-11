//! The built-in functions and modules (A17 §12): Python's everyday ones, `math`, `random` (the block's own seeded
//! stream) and `simulation`, and the methods of texts, lists, tuples and dictionaries. Functions that need the running
//! program (printing, random numbers, `now()`, sort keys) are finished by the interpreter; the rest are pure.

use std::sync::Arc;

use crate::ops::{Fault, MAX_ITEMS, Res, order, position};
use crate::units::Dim;
use crate::value::{Dict, Iter, Value, lock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Print,
    Warn,
    Error,
    Len,
    Range,
    Abs,
    Min,
    Max,
    Sum,
    Round,
    Int,
    Float,
    Str,
    Bool,
    List,
    Dict,
    Tuple,
    Sorted,
    Reversed,
    Enumerate,
    Zip,
    Any,
    All,
    Repr,
    Now,
    Math,
    Random,
    Simulation,
    MathSqrt,
    MathSin,
    MathCos,
    MathTan,
    MathAtan2,
    MathFloor,
    MathCeil,
    MathHypot,
    MathIsclose,
    MathExp,
    MathLog,
    RandomRandom,
    RandomUniform,
    RandomNormal,
    RandomExponential,
    RandomTriangular,
    RandomRandint,
    RandomChance,
    RandomChoice,
    SimStop,
}

/// The names code can use without defining them, with their help line for the editor.
pub const GLOBALS: [(&str, Builtin, &str); 28] = [
    ("print", Builtin::Print, "print(a, b, …): writes the values to the run's console, with the time"),
    ("warn", Builtin::Warn, "warn(a, b, …): a warning in the console and the run's issues"),
    ("error", Builtin::Error, "error(message): stops the run with this message"),
    ("len", Builtin::Len, "len(x): items of a list, tuple, dictionary or text"),
    ("range", Builtin::Range, "range(stop) / range(start, stop[, step]): whole numbers"),
    ("abs", Builtin::Abs, "abs(x): size of a number or quantity"),
    ("min", Builtin::Min, "min(a, b, …) or min(list, key=f)"),
    ("max", Builtin::Max, "max(a, b, …) or max(list, key=f)"),
    ("sum", Builtin::Sum, "sum(list): total of numbers or of quantities of one kind"),
    ("round", Builtin::Round, "round(x[, digits]): halves to even, as Python"),
    ("int", Builtin::Int, "int(x): whole number of a number or text"),
    ("float", Builtin::Float, "float(x): number of a number or text"),
    ("str", Builtin::Str, "str(x): x as text"),
    ("bool", Builtin::Bool, "bool(x): x's truth"),
    ("list", Builtin::List, "list(x): a new list of x's items"),
    ("dict", Builtin::Dict, "dict() or dict(pairs): a new dictionary"),
    ("tuple", Builtin::Tuple, "tuple(x): a tuple of x's items"),
    ("sorted", Builtin::Sorted, "sorted(x, key=f, reverse=False): a sorted list"),
    ("reversed", Builtin::Reversed, "reversed(x): x's items backwards"),
    ("enumerate", Builtin::Enumerate, "enumerate(x, start=0): (index, item) pairs"),
    ("zip", Builtin::Zip, "zip(a, b, …): items side by side"),
    ("any", Builtin::Any, "any(x): some item is true"),
    ("all", Builtin::All, "all(x): every item is true"),
    ("repr", Builtin::Repr, "repr(x): x as code would write it"),
    ("now", Builtin::Now, "now(): the simulation time, a time quantity"),
    ("math", Builtin::Math, "math.sqrt, sin, cos, tan, atan2, floor, ceil, hypot, isclose, exp, log, pi, inf"),
    ("random", Builtin::Random, "random.random, uniform, normal, exponential, triangular, randint, chance, choice"),
    ("simulation", Builtin::Simulation, "simulation.mode, simulation.seed, simulation.stop(reason)"),
];

const MATH: [(&str, Builtin); 11] = [
    ("sqrt", Builtin::MathSqrt),
    ("sin", Builtin::MathSin),
    ("cos", Builtin::MathCos),
    ("tan", Builtin::MathTan),
    ("atan2", Builtin::MathAtan2),
    ("floor", Builtin::MathFloor),
    ("ceil", Builtin::MathCeil),
    ("hypot", Builtin::MathHypot),
    ("isclose", Builtin::MathIsclose),
    ("exp", Builtin::MathExp),
    ("log", Builtin::MathLog),
];

const RANDOM: [(&str, Builtin); 9] = [
    ("random", Builtin::RandomRandom),
    ("uniform", Builtin::RandomUniform),
    ("normal", Builtin::RandomNormal),
    ("gauss", Builtin::RandomNormal),
    ("exponential", Builtin::RandomExponential),
    ("triangular", Builtin::RandomTriangular),
    ("randint", Builtin::RandomRandint),
    ("chance", Builtin::RandomChance),
    ("choice", Builtin::RandomChoice),
];

/// The methods of each kind of value, for the checker's messages and the editor.
pub const LIST_METHODS: [&str; 11] = ["append", "extend", "insert", "pop", "remove", "index", "count", "clear", "sort", "reverse", "copy"];
pub const DICT_METHODS: [&str; 9] = ["get", "keys", "values", "items", "pop", "setdefault", "update", "clear", "copy"];
pub const STR_METHODS: [&str; 14] = [
    "upper",
    "lower",
    "strip",
    "lstrip",
    "rstrip",
    "startswith",
    "endswith",
    "split",
    "join",
    "replace",
    "find",
    "count",
    "isdigit",
    "zfill",
];

pub fn global(name: &str) -> Option<Builtin> {
    GLOBALS.iter().find(|(n, ..)| *n == name).map(|(_, b, _)| *b)
}

/// `math.x`, `random.x`, `simulation.x` (values that need the host are the interpreter's).
pub fn module_member(module: Builtin, name: &str) -> Option<Value> {
    match module {
        Builtin::Math => match name {
            "pi" => Some(Value::Float(std::f64::consts::PI)),
            "e" => Some(Value::Float(std::f64::consts::E)),
            "inf" => Some(Value::Float(f64::INFINITY)),
            _ => MATH.iter().find(|(n, _)| *n == name).map(|(_, b)| Value::Builtin(*b)),
        },
        Builtin::Random => RANDOM.iter().find(|(n, _)| *n == name).map(|(_, b)| Value::Builtin(*b)),
        Builtin::Simulation if name == "stop" => Some(Value::Builtin(Builtin::SimStop)),
        _ => None,
    }
}

/// The names a module offers, for the editor.
pub fn module_names(module: Builtin) -> Vec<&'static str> {
    match module {
        Builtin::Math => MATH.iter().map(|(n, _)| *n).chain(["pi", "e", "inf"]).collect(),
        Builtin::Random => RANDOM.iter().map(|(n, _)| *n).collect(),
        Builtin::Simulation => vec!["mode", "seed", "stop"],
        _ => Vec::new(),
    }
}

/// Every item of an iterable, in order.
pub fn items(v: &Value) -> Res<Vec<Value>> {
    Ok(match v {
        Value::List(l) => lock(l).clone(),
        Value::Tuple(t) => t.as_ref().clone(),
        Value::Str(s) => s.chars().map(|c| Value::str(&c.to_string())).collect(),
        Value::Dict(d) => lock(d).keys(),
        Value::Range(a, b, s) => {
            let n = crate::ops::range_len(*a, *b, *s);
            if n > MAX_ITEMS {
                return Err(Fault::new("tooLarge", format!("a range of more than {MAX_ITEMS} numbers can't become a list")));
            }
            (0..n).map(|i| Value::Int(a + s * i as i64)).collect()
        }
        Value::Iter(it) => {
            let mut out = Vec::new();
            for v in lock(it).by_ref() {
                if out.len() >= MAX_ITEMS {
                    return Err(Fault::new("tooLarge", format!("more than {MAX_ITEMS} items")));
                }
                out.push(v);
            }
            out
        }
        other => return Err(Fault::types(format!("a {} has no items to go through", other.type_name()))),
    })
}

/// What `for` walks through.
pub fn iterate(v: &Value) -> Res<Iter> {
    Ok(match v {
        Value::List(l) => Iter::List(l.clone(), 0),
        Value::Range(a, b, s) => Iter::Range(*a, *b, *s),
        Value::Iter(it) => {
            // An iterator walks once: take what is left.
            let rest = items(&Value::Iter(it.clone()))?;
            Iter::Items(rest, 0)
        }
        other => Iter::Items(items(other)?, 0),
    })
}

fn arity(name: &str, args: &[Value], min: usize, max: usize) -> Res<()> {
    if args.len() < min || args.len() > max {
        let want = if min == max { format!("{min}") } else { format!("{min} to {max}") };
        return Err(Fault::new("wrongArguments", format!("{name}() takes {want} arguments, not {}", args.len())));
    }
    Ok(())
}

fn no_kwargs(name: &str, kwargs: &[(Arc<str>, Value)]) -> Res<()> {
    match kwargs.first() {
        Some((k, _)) => Err(Fault::new("wrongArguments", format!("{name}() takes no argument `{k}`"))),
        None => Ok(()),
    }
}

fn kw<'a>(kwargs: &'a [(Arc<str>, Value)], name: &str) -> Option<&'a Value> {
    kwargs.iter().find(|(k, _)| k.as_ref() == name).map(|(_, v)| v)
}

fn only_kwargs(fname: &str, kwargs: &[(Arc<str>, Value)], allowed: &[&str]) -> Res<()> {
    for (k, _) in kwargs {
        if !allowed.contains(&k.as_ref()) {
            return Err(Fault::new("wrongArguments", format!("{fname}() takes no argument `{k}`")));
        }
    }
    Ok(())
}

fn int_arg(v: &Value, what: &str) -> Res<i64> {
    match v {
        Value::Int(i) => Ok(*i),
        Value::Bool(b) => Ok(i64::from(*b)),
        other => Err(Fault::types(format!("{what} must be a whole number, not a {}", other.type_name()))),
    }
}

fn number(v: &Value, what: &str) -> Res<f64> {
    v.as_f64().ok_or_else(|| Fault::types(format!("{what} needs a plain number, not a {}", v.type_name())))
}

/// The pure built-ins (those that need neither the running program nor the engine).
pub fn pure(b: Builtin, args: &[Value], kwargs: &[(Arc<str>, Value)]) -> Res<Value> {
    match b {
        Builtin::Len => {
            no_kwargs("len", kwargs)?;
            arity("len", args, 1, 1)?;
            let n = match &args[0] {
                Value::List(l) => lock(l).len(),
                Value::Tuple(t) => t.len(),
                Value::Dict(d) => lock(d).len(),
                Value::Str(s) => s.chars().count(),
                Value::Range(a, b, s) => crate::ops::range_len(*a, *b, *s),
                other => return Err(Fault::types(format!("a {} has no length", other.type_name()))),
            };
            Ok(Value::Int(i64::try_from(n).unwrap_or(i64::MAX)))
        }
        Builtin::Range => {
            no_kwargs("range", kwargs)?;
            arity("range", args, 1, 3)?;
            let n: Vec<i64> = args.iter().map(|a| int_arg(a, "range()'s arguments")).collect::<Res<_>>()?;
            let (a, b, s) = match n.as_slice() {
                [b] => (0, *b, 1),
                [a, b] => (*a, *b, 1),
                [a, b, s] => (*a, *b, *s),
                _ => (0, 0, 1),
            };
            if s == 0 {
                return Err(Fault::new("wrongArguments", "range()'s step can't be 0"));
            }
            Ok(Value::Range(a, b, s))
        }
        Builtin::Abs => {
            arity("abs", args, 1, 1)?;
            match &args[0] {
                Value::Int(i) => i.checked_abs().map(Value::Int).ok_or_else(|| Fault::new("tooLarge", "abs() overflows")),
                Value::Bool(x) => Ok(Value::Int(i64::from(*x))),
                Value::Float(x) => Ok(Value::Float(x.abs())),
                Value::Qty(x, d) => Ok(Value::Qty(x.abs(), *d)),
                other => Err(Fault::types(format!("abs() needs a number, not a {}", other.type_name()))),
            }
        }
        Builtin::Sum => {
            only_kwargs("sum", kwargs, &["start"])?;
            arity("sum", args, 1, 2)?;
            let list = items(&args[0])?;
            let start = args.get(1).or_else(|| kw(kwargs, "start")).cloned();
            let mut it = list.into_iter();
            let mut total = match start {
                Some(s) => s,
                None => match it.next() {
                    Some(first) => first,
                    None => return Ok(Value::Int(0)),
                },
            };
            for v in it {
                total = crate::ops::binary(crate::ast::BinOp::Add, &total, &v)?;
            }
            Ok(total)
        }
        Builtin::Round => {
            no_kwargs("round", kwargs)?;
            arity("round", args, 1, 2)?;
            if let Value::Qty(_, d) = &args[0] {
                return Err(Fault::new(
                    "unitMismatch",
                    format!("round a {} in the unit you mean: round(x / mm) gives whole millimetres", d.name()),
                ));
            }
            let x = number(&args[0], "round()")?;
            match args.get(1) {
                None => {
                    if let Value::Int(i) = args[0] {
                        return Ok(Value::Int(i));
                    }
                    let r = x.round_ties_even();
                    if r.abs() > 9.0e18 || r.is_nan() {
                        return Err(Fault::new("tooLarge", "round() of a number too large for a whole number"));
                    }
                    Ok(Value::Int(r as i64))
                }
                Some(d) => {
                    let digits = int_arg(d, "round()'s digits")?.clamp(-15, 15);
                    let f = 10f64.powi(digits as i32);
                    Ok(Value::Float((x * f).round_ties_even() / f))
                }
            }
        }
        Builtin::Int => {
            no_kwargs("int", kwargs)?;
            arity("int", args, 0, 1)?;
            match args.first() {
                None => Ok(Value::Int(0)),
                Some(Value::Int(i)) => Ok(Value::Int(*i)),
                Some(Value::Bool(b)) => Ok(Value::Int(i64::from(*b))),
                Some(Value::Float(x)) => {
                    if !x.is_finite() || x.abs() > 9.0e18 {
                        return Err(Fault::new("tooLarge", "int() of a number too large or not finite"));
                    }
                    Ok(Value::Int(x.trunc() as i64))
                }
                Some(Value::Str(s)) => s
                    .trim()
                    .replace('_', "")
                    .parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| Fault::types(format!("{:?} is not a whole number", s.as_ref()))),
                Some(Value::Qty(_, d)) => {
                    Err(Fault::new("unitMismatch", format!("int() of a {}: divide by its unit first, int(x / mm)", d.name())))
                }
                Some(other) => Err(Fault::types(format!("int() of a {}", other.type_name()))),
            }
        }
        Builtin::Float => {
            no_kwargs("float", kwargs)?;
            arity("float", args, 0, 1)?;
            match args.first() {
                None => Ok(Value::Float(0.0)),
                Some(Value::Str(s)) => {
                    let t = s.trim().to_ascii_lowercase();
                    match t.as_str() {
                        "inf" | "infinity" => Ok(Value::Float(f64::INFINITY)),
                        "-inf" | "-infinity" => Ok(Value::Float(f64::NEG_INFINITY)),
                        _ => t
                            .replace('_', "")
                            .parse::<f64>()
                            .map(Value::Float)
                            .map_err(|_| Fault::types(format!("{:?} is not a number", s.as_ref()))),
                    }
                }
                Some(Value::Qty(_, d)) => {
                    Err(Fault::new("unitMismatch", format!("float() of a {}: divide by its unit first, x / mm", d.name())))
                }
                Some(v) => Ok(Value::Float(number(v, "float()")?)),
            }
        }
        Builtin::Str => {
            arity("str", args, 0, 1)?;
            Ok(Value::str(&args.first().map(Value::to_text).unwrap_or_default()))
        }
        Builtin::Repr => {
            arity("repr", args, 1, 1)?;
            Ok(Value::str(&args[0].repr()))
        }
        Builtin::Bool => {
            arity("bool", args, 0, 1)?;
            Ok(Value::Bool(args.first().is_some_and(Value::truthy)))
        }
        Builtin::List => {
            arity("list", args, 0, 1)?;
            Ok(Value::list(match args.first() {
                Some(v) => items(v)?,
                None => Vec::new(),
            }))
        }
        Builtin::Tuple => {
            arity("tuple", args, 0, 1)?;
            Ok(Value::tuple(match args.first() {
                Some(v) => items(v)?,
                None => Vec::new(),
            }))
        }
        Builtin::Dict => {
            arity("dict", args, 0, 1)?;
            let mut d = Dict::default();
            if let Some(src) = args.first() {
                if let Value::Dict(other) = src {
                    d = lock(other).clone();
                } else {
                    for pair in items(src)? {
                        let kv = items(&pair)?;
                        let [k, v] = kv.as_slice() else {
                            return Err(Fault::types("dict() takes pairs: [(key, value), …]"));
                        };
                        d.set(k.clone(), v.clone()).map_err(Fault::types)?;
                    }
                }
            }
            for (k, v) in kwargs {
                d.set(Value::Str(k.clone()), v.clone()).map_err(Fault::types)?;
            }
            Ok(Value::dict(d))
        }
        Builtin::Reversed => {
            arity("reversed", args, 1, 1)?;
            let mut v = items(&args[0])?;
            v.reverse();
            Ok(Value::Iter(Arc::new(std::sync::Mutex::new(Iter::Items(v, 0)))))
        }
        Builtin::Enumerate => {
            only_kwargs("enumerate", kwargs, &["start"])?;
            arity("enumerate", args, 1, 2)?;
            let start = match args.get(1).or_else(|| kw(kwargs, "start")) {
                Some(s) => int_arg(s, "enumerate()'s start")?,
                None => 0,
            };
            Ok(Value::Iter(Arc::new(std::sync::Mutex::new(Iter::Enumerate(Box::new(iterate(&args[0])?), start)))))
        }
        Builtin::Zip => {
            no_kwargs("zip", kwargs)?;
            let inners = args.iter().map(iterate).collect::<Res<Vec<_>>>()?;
            Ok(Value::Iter(Arc::new(std::sync::Mutex::new(Iter::Zip(inners)))))
        }
        Builtin::Any => {
            arity("any", args, 1, 1)?;
            Ok(Value::Bool(items(&args[0])?.iter().any(Value::truthy)))
        }
        Builtin::All => {
            arity("all", args, 1, 1)?;
            Ok(Value::Bool(items(&args[0])?.iter().all(Value::truthy)))
        }
        Builtin::MathSqrt => {
            arity("math.sqrt", args, 1, 1)?;
            let (x, d) = quantity(&args[0], "math.sqrt()")?;
            if x < 0.0 {
                return Err(Fault::new("notANumber", "math.sqrt() of a negative number"));
            }
            let d = d.sqrt().ok_or_else(|| Fault::new("unitMismatch", format!("the square root of a {} has no unit", d.name())))?;
            Ok(Value::qty(x.sqrt(), d))
        }
        Builtin::MathSin | Builtin::MathCos | Builtin::MathTan => {
            arity("math.sin", args, 1, 1)?;
            let (x, d) = quantity(&args[0], "math.sin()")?;
            if d != Dim::ANGLE && !d.is_none() {
                return Err(Fault::new("unitMismatch", format!("sin, cos and tan take an angle, not a {}", d.name())));
            }
            Ok(Value::Float(match b {
                Builtin::MathSin => x.sin(),
                Builtin::MathCos => x.cos(),
                _ => x.tan(),
            }))
        }
        Builtin::MathAtan2 => {
            arity("math.atan2", args, 2, 2)?;
            let ((y, dy), (x, dx)) = (quantity(&args[0], "math.atan2()")?, quantity(&args[1], "math.atan2()")?);
            if dy != dx {
                return Err(crate::ops::unit_mismatch("compare", dy, dx));
            }
            Ok(Value::Qty(y.atan2(x), Dim::ANGLE))
        }
        Builtin::MathFloor | Builtin::MathCeil => {
            arity("math.floor", args, 1, 1)?;
            if let Value::Int(i) = args[0] {
                return Ok(Value::Int(i));
            }
            let x = number(&args[0], "math.floor() and math.ceil()")?;
            let r = if b == Builtin::MathFloor { x.floor() } else { x.ceil() };
            if !r.is_finite() || r.abs() > 9.0e18 {
                return Err(Fault::new("tooLarge", "too large for a whole number"));
            }
            Ok(Value::Int(r as i64))
        }
        Builtin::MathHypot => {
            let mut d0 = None;
            let mut total = 0.0;
            for a in args {
                let (x, d) = quantity(a, "math.hypot()")?;
                if *d0.get_or_insert(d) != d {
                    return Err(crate::ops::unit_mismatch("combine", d0.unwrap_or(d), d));
                }
                total += x * x;
            }
            Ok(Value::qty(total.sqrt(), d0.unwrap_or(Dim::NONE)))
        }
        Builtin::MathIsclose => {
            only_kwargs("math.isclose", kwargs, &["rel_tol", "abs_tol"])?;
            arity("math.isclose", args, 2, 2)?;
            let ((a, da), (c, dc)) = (quantity(&args[0], "math.isclose()")?, quantity(&args[1], "math.isclose()")?);
            if da != dc {
                return Err(crate::ops::unit_mismatch("compare", da, dc));
            }
            let rel = kw(kwargs, "rel_tol").map(|v| number(v, "rel_tol")).transpose()?.unwrap_or(1e-9);
            let abs = match kw(kwargs, "abs_tol") {
                Some(v) => quantity(v, "abs_tol")?.0,
                None => 0.0,
            };
            Ok(Value::Bool((a - c).abs() <= (rel * a.abs().max(c.abs())).max(abs)))
        }
        Builtin::MathExp => {
            arity("math.exp", args, 1, 1)?;
            Ok(Value::Float(number(&args[0], "math.exp()")?.exp()))
        }
        Builtin::MathLog => {
            arity("math.log", args, 1, 2)?;
            let x = number(&args[0], "math.log()")?;
            if x <= 0.0 {
                return Err(Fault::new("notANumber", "math.log() of a number that is not positive"));
            }
            Ok(Value::Float(match args.get(1) {
                Some(base) => x.ln() / number(base, "math.log()'s base")?.ln(),
                None => x.ln(),
            }))
        }
        other => Err(Fault::new("internal", format!("{other:?} is not a pure built-in"))),
    }
}

/// A number or quantity: its SI value and dimension.
pub fn quantity(v: &Value, what: &str) -> Res<(f64, Dim)> {
    v.as_quantity().ok_or_else(|| Fault::types(format!("{what} needs a number or quantity, not a {}", v.type_name())))
}

/// min() and max() over arguments or one iterable, with the keys already worked out (one per item) when given.
pub fn extreme(max: bool, candidates: Vec<Value>, keys: Option<Vec<Value>>) -> Res<Value> {
    if candidates.is_empty() {
        return Err(Fault::new("emptyList", format!("{}() of nothing", if max { "max" } else { "min" })));
    }
    let keys = keys.unwrap_or_else(|| candidates.clone());
    let mut best = 0;
    for i in 1..candidates.len() {
        let o = order(&keys[i], &keys[best])?;
        if (max && o == std::cmp::Ordering::Greater) || (!max && o == std::cmp::Ordering::Less) {
            best = i;
        }
    }
    Ok(candidates[best].clone())
}

/// Sorts items by their keys (stable, as Python's sort).
pub fn sort_by_keys(items: Vec<Value>, keys: Vec<Value>, reverse: bool) -> Res<Vec<Value>> {
    let mut pairs: Vec<(Value, Value)> = keys.into_iter().zip(items).collect();
    let mut fault = None;
    pairs.sort_by(|a, b| match order(&a.0, &b.0) {
        Ok(o) => {
            if reverse {
                o.reverse()
            } else {
                o
            }
        }
        Err(e) => {
            fault.get_or_insert(e);
            std::cmp::Ordering::Equal
        }
    });
    match fault {
        Some(e) => Err(e),
        None => Ok(pairs.into_iter().map(|(_, v)| v).collect()),
    }
}

/// A method of a text, list, tuple or dictionary (`sort` with a key is the interpreter's).
pub fn method(recv: &Value, name: &str, args: &[Value], kwargs: &[(Arc<str>, Value)]) -> Res<Value> {
    match recv {
        Value::List(l) => list_method(l, name, args, kwargs),
        Value::Dict(d) => {
            no_kwargs(name, kwargs)?;
            dict_method(d, name, args)
        }
        Value::Str(s) => {
            no_kwargs(name, kwargs)?;
            str_method(s, name, args)
        }
        Value::Tuple(t) => match name {
            "count" => {
                arity("count", args, 1, 1)?;
                Ok(Value::Int(t.iter().filter(|v| v.equals(&args[0])).count() as i64))
            }
            "index" => {
                arity("index", args, 1, 1)?;
                t.iter()
                    .position(|v| v.equals(&args[0]))
                    .map(|i| Value::Int(i as i64))
                    .ok_or_else(|| Fault::new("notFound", format!("{} is not in the tuple", args[0].repr())))
            }
            _ => Err(no_method(recv, name)),
        },
        _ => Err(no_method(recv, name)),
    }
}

pub fn no_method(recv: &Value, name: &str) -> Fault {
    Fault::new("unknownAttribute", format!("a {} has no attribute `{name}`", recv.type_name()))
}

/// Whether `name` is a method of this kind of value.
pub fn has_method(recv: &Value, name: &str) -> bool {
    match recv {
        Value::List(_) => LIST_METHODS.contains(&name),
        Value::Dict(_) => DICT_METHODS.contains(&name),
        Value::Str(_) => STR_METHODS.contains(&name),
        Value::Tuple(_) => matches!(name, "count" | "index"),
        _ => false,
    }
}

fn list_method(l: &crate::value::ListRef, name: &str, args: &[Value], kwargs: &[(Arc<str>, Value)]) -> Res<Value> {
    if name != "sort" {
        no_kwargs(name, kwargs)?;
    }
    let full = || Fault::new("tooLarge", format!("a list holds at most {MAX_ITEMS} items"));
    match name {
        "append" => {
            arity("append", args, 1, 1)?;
            let mut v = lock(l);
            if v.len() >= MAX_ITEMS {
                return Err(full());
            }
            v.push(args[0].clone());
            Ok(Value::None)
        }
        "extend" => {
            arity("extend", args, 1, 1)?;
            let more = items(&args[0])?;
            let mut v = lock(l);
            if v.len() + more.len() > MAX_ITEMS {
                return Err(full());
            }
            v.extend(more);
            Ok(Value::None)
        }
        "insert" => {
            arity("insert", args, 2, 2)?;
            let i = int_arg(&args[0], "insert()'s index")?;
            let mut v = lock(l);
            if v.len() >= MAX_ITEMS {
                return Err(full());
            }
            let n = v.len() as i64;
            let at = if i < 0 { (i + n).max(0) } else { i.min(n) } as usize;
            v.insert(at, args[1].clone());
            Ok(Value::None)
        }
        "pop" => {
            arity("pop", args, 0, 1)?;
            let mut v = lock(l);
            if v.is_empty() {
                return Err(Fault::new("emptyList", "pop() from an empty list"));
            }
            let i = match args.first() {
                Some(i) => position(i, v.len())?,
                None => v.len() - 1,
            };
            Ok(v.remove(i))
        }
        "remove" => {
            arity("remove", args, 1, 1)?;
            let snapshot = lock(l).clone();
            let i = snapshot
                .iter()
                .position(|x| x.equals(&args[0]))
                .ok_or_else(|| Fault::new("notFound", format!("{} is not in the list", args[0].repr())))?;
            let mut v = lock(l);
            if i < v.len() {
                v.remove(i);
            }
            Ok(Value::None)
        }
        "index" => {
            arity("index", args, 1, 1)?;
            let snapshot = lock(l).clone();
            snapshot
                .iter()
                .position(|x| x.equals(&args[0]))
                .map(|i| Value::Int(i as i64))
                .ok_or_else(|| Fault::new("notFound", format!("{} is not in the list", args[0].repr())))
        }
        "count" => {
            arity("count", args, 1, 1)?;
            let snapshot = lock(l).clone();
            Ok(Value::Int(snapshot.iter().filter(|x| x.equals(&args[0])).count() as i64))
        }
        "clear" => {
            arity("clear", args, 0, 0)?;
            lock(l).clear();
            Ok(Value::None)
        }
        "reverse" => {
            arity("reverse", args, 0, 0)?;
            lock(l).reverse();
            Ok(Value::None)
        }
        "copy" => {
            arity("copy", args, 0, 0)?;
            Ok(Value::list(lock(l).clone()))
        }
        "sort" => {
            only_kwargs("sort", kwargs, &["reverse"])?;
            arity("sort", args, 0, 0)?;
            let reverse = kw(kwargs, "reverse").is_some_and(Value::truthy);
            let snapshot = lock(l).clone();
            let sorted = sort_by_keys(snapshot.clone(), snapshot, reverse)?;
            *lock(l) = sorted;
            Ok(Value::None)
        }
        _ => Err(no_method(&Value::List(l.clone()), name)),
    }
}

fn dict_method(d: &crate::value::DictRef, name: &str, args: &[Value]) -> Res<Value> {
    match name {
        "get" => {
            arity("get", args, 1, 2)?;
            let found = lock(d).get(&args[0]).map_err(Fault::types)?;
            Ok(found.unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::None)))
        }
        "keys" => Ok(Value::list(lock(d).keys())),
        "values" => Ok(Value::list(lock(d).items().into_iter().map(|(_, v)| v).collect())),
        "items" => Ok(Value::list(lock(d).items().into_iter().map(|(k, v)| Value::tuple(vec![k, v])).collect())),
        "pop" => {
            arity("pop", args, 1, 2)?;
            let removed = lock(d).remove(&args[0]).map_err(Fault::types)?;
            match (removed, args.get(1)) {
                (Some(v), _) => Ok(v),
                (None, Some(default)) => Ok(default.clone()),
                (None, None) => Err(Fault::new("keyNotFound", format!("the dictionary has no key {}", args[0].repr()))),
            }
        }
        "setdefault" => {
            arity("setdefault", args, 1, 2)?;
            let mut dict = lock(d);
            if let Some(v) = dict.get(&args[0]).map_err(Fault::types)? {
                return Ok(v);
            }
            let v = args.get(1).cloned().unwrap_or(Value::None);
            dict.set(args[0].clone(), v.clone()).map_err(Fault::types)?;
            Ok(v)
        }
        "update" => {
            arity("update", args, 1, 1)?;
            let Value::Dict(other) = &args[0] else { return Err(Fault::types("update() takes a dictionary")) };
            let pairs = lock(other).items();
            let mut dict = lock(d);
            for (k, v) in pairs {
                dict.set(k, v).map_err(Fault::types)?;
            }
            Ok(Value::None)
        }
        "clear" => {
            lock(d).clear();
            Ok(Value::None)
        }
        "copy" => Ok(Value::dict(lock(d).clone())),
        _ => Err(no_method(&Value::Dict(d.clone()), name)),
    }
}

fn str_arg<'a>(v: &'a Value, what: &str) -> Res<&'a str> {
    match v {
        Value::Str(s) => Ok(s),
        other => Err(Fault::types(format!("{what} takes a text, not a {}", other.type_name()))),
    }
}

fn str_method(s: &str, name: &str, args: &[Value]) -> Res<Value> {
    let text = |t: &str| Ok(Value::str(t));
    match name {
        "upper" => text(&s.to_uppercase()),
        "lower" => text(&s.to_lowercase()),
        "strip" => text(s.trim()),
        "lstrip" => text(s.trim_start()),
        "rstrip" => text(s.trim_end()),
        "isdigit" => Ok(Value::Bool(!s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))),
        "startswith" => {
            arity(name, args, 1, 1)?;
            Ok(Value::Bool(s.starts_with(str_arg(&args[0], name)?)))
        }
        "endswith" => {
            arity(name, args, 1, 1)?;
            Ok(Value::Bool(s.ends_with(str_arg(&args[0], name)?)))
        }
        "find" => {
            arity(name, args, 1, 1)?;
            let needle = str_arg(&args[0], name)?;
            Ok(Value::Int(s.find(needle).map_or(-1, |b| s[..b].chars().count() as i64)))
        }
        "count" => {
            arity(name, args, 1, 1)?;
            let needle = str_arg(&args[0], name)?;
            Ok(Value::Int(if needle.is_empty() { s.chars().count() as i64 + 1 } else { s.matches(needle).count() as i64 }))
        }
        "replace" => {
            arity(name, args, 2, 2)?;
            let (a, b) = (str_arg(&args[0], name)?, str_arg(&args[1], name)?);
            if a.is_empty() {
                return Err(Fault::types("replace() of an empty text"));
            }
            let out = s.replace(a, b);
            if out.chars().count() > MAX_ITEMS {
                return Err(Fault::new("tooLarge", "the text grows too long"));
            }
            text(&out)
        }
        "split" => {
            arity(name, args, 0, 1)?;
            let parts: Vec<Value> = match args.first() {
                None | Some(Value::None) => s.split_whitespace().map(Value::str).collect(),
                Some(sep) => {
                    let sep = str_arg(sep, name)?;
                    if sep.is_empty() {
                        return Err(Fault::types("split() with an empty separator"));
                    }
                    s.split(sep).map(Value::str).collect()
                }
            };
            Ok(Value::list(parts))
        }
        "join" => {
            arity(name, args, 1, 1)?;
            let parts = items(&args[0])?;
            let mut out = Vec::with_capacity(parts.len());
            for p in &parts {
                out.push(str_arg(p, "join()")?.to_string());
            }
            let joined = out.join(s);
            if joined.chars().count() > MAX_ITEMS {
                return Err(Fault::new("tooLarge", "the text grows too long"));
            }
            text(&joined)
        }
        "zfill" => {
            arity(name, args, 1, 1)?;
            let w = usize::try_from(int_arg(&args[0], "zfill()")?.clamp(0, 1000)).unwrap_or(0);
            let n = s.chars().count();
            if n >= w {
                return text(s);
            }
            let (sign, rest) = if let Some(r) = s.strip_prefix('-') { ("-", r) } else { ("", s) };
            text(&format!("{sign}{}{rest}", "0".repeat(w - n)))
        }
        _ => Err(no_method(&Value::str(s), name)),
    }
}

/// The block's own random stream: PCG32 (XSH-RR), the same generator as the engines'.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
    increment: u64,
}

impl Rng {
    pub fn new(seed: u64, stream: u64) -> Rng {
        let mut r = Rng { state: 0, increment: (stream << 1) | 1 };
        r.next_u32();
        r.state = r.state.wrapping_add(seed);
        r.next_u32();
        r
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(self.increment);
        let x = (((old >> 18) ^ old) >> 27) as u32;
        x.rotate_right((old >> 59) as u32)
    }

    /// Uniform in [0, 1), with 53 bits.
    pub fn next_f64(&mut self) -> f64 {
        let hi = u64::from(self.next_u32()) >> 5;
        let lo = u64::from(self.next_u32()) >> 6;
        ((hi << 26) | lo) as f64 / (1u64 << 53) as f64
    }

    /// A standard normal number (Box–Muller).
    pub fn normal(&mut self) -> f64 {
        let u1 = 1.0 - self.next_f64();
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// A random built-in with the block's stream.
pub fn random(b: Builtin, rng: &mut Rng, args: &[Value], kwargs: &[(Arc<str>, Value)]) -> Res<Value> {
    no_kwargs("random", kwargs)?;
    let two = |name: &str| -> Res<((f64, Dim), (f64, Dim))> {
        arity(name, args, 2, 2)?;
        let (a, b) = (quantity(&args[0], name)?, quantity(&args[1], name)?);
        if a.1 != b.1 {
            return Err(crate::ops::unit_mismatch("mix", a.1, b.1));
        }
        Ok((a, b))
    };
    match b {
        Builtin::RandomRandom => {
            arity("random.random", args, 0, 0)?;
            Ok(Value::Float(rng.next_f64()))
        }
        Builtin::RandomUniform => {
            let ((a, d), (c, _)) = two("random.uniform")?;
            Ok(Value::qty(a + (c - a) * rng.next_f64(), d))
        }
        Builtin::RandomNormal => {
            let ((mean, d), (sd, _)) = two("random.normal")?;
            if sd < 0.0 {
                return Err(Fault::new("wrongArguments", "random.normal()'s standard deviation is negative"));
            }
            Ok(Value::qty(mean + sd * rng.normal(), d))
        }
        Builtin::RandomExponential => {
            arity("random.exponential", args, 1, 1)?;
            let (mean, d) = quantity(&args[0], "random.exponential()")?;
            if mean <= 0.0 {
                return Err(Fault::new("wrongArguments", "random.exponential()'s mean must be positive"));
            }
            Ok(Value::qty(-mean * (1.0 - rng.next_f64()).ln(), d))
        }
        Builtin::RandomTriangular => {
            arity("random.triangular", args, 3, 3)?;
            let (lo, d) = quantity(&args[0], "random.triangular()")?;
            let (hi, d2) = quantity(&args[1], "random.triangular()")?;
            let (mode, d3) = quantity(&args[2], "random.triangular()")?;
            if d != d2 || d != d3 {
                return Err(crate::ops::unit_mismatch("mix", d, if d != d2 { d2 } else { d3 }));
            }
            if !(lo <= mode && mode <= hi) || lo == hi {
                return Err(Fault::new("wrongArguments", "random.triangular(low, high, mode) needs low ≤ mode ≤ high and low < high"));
            }
            let u = rng.next_f64();
            let c = (mode - lo) / (hi - lo);
            let x = if u < c { lo + ((hi - lo) * (mode - lo) * u).sqrt() } else { hi - ((hi - lo) * (hi - mode) * (1.0 - u)).sqrt() };
            Ok(Value::qty(x, d))
        }
        Builtin::RandomRandint => {
            arity("random.randint", args, 2, 2)?;
            let (a, c) = (int_arg(&args[0], "random.randint()")?, int_arg(&args[1], "random.randint()")?);
            if a > c {
                return Err(Fault::new("wrongArguments", "random.randint(a, b) needs a ≤ b"));
            }
            let span = (c - a).saturating_add(1) as f64;
            Ok(Value::Int(a + ((rng.next_f64() * span).floor() as i64).min(c - a)))
        }
        Builtin::RandomChance => {
            arity("random.chance", args, 1, 1)?;
            let p = number(&args[0], "random.chance()")?;
            Ok(Value::Bool(rng.next_f64() < p))
        }
        Builtin::RandomChoice => {
            arity("random.choice", args, 1, 1)?;
            let all = items(&args[0])?;
            if all.is_empty() {
                return Err(Fault::new("emptyList", "random.choice() of an empty list"));
            }
            let k = ((rng.next_f64() * all.len() as f64).floor() as usize).min(all.len() - 1);
            Ok(all[k].clone())
        }
        other => Err(Fault::new("internal", format!("{other:?} is not a random built-in"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(b: Builtin, args: &[Value]) -> Res<Value> {
        pure(b, args, &[])
    }

    #[test]
    fn pythons_everyday_functions() {
        assert_eq!(call(Builtin::Len, &[Value::str("palé")]).unwrap().repr(), "4");
        assert_eq!(call(Builtin::List, &[Value::Range(0, 10, 3)]).unwrap().repr(), "[0, 3, 6, 9]");
        assert_eq!(call(Builtin::Round, &[Value::Float(2.5)]).unwrap().repr(), "2");
        assert_eq!(call(Builtin::Round, &[Value::Float(0.125), Value::Int(2)]).unwrap().repr(), "0.12");
        assert_eq!(call(Builtin::Int, &[Value::str(" 42 ")]).unwrap().repr(), "42");
        let quantities = Value::list(vec![Value::Qty(1.0, Dim::LENGTH), Value::Qty(0.5, Dim::LENGTH)]);
        assert_eq!(call(Builtin::Sum, &[quantities]).unwrap().to_text(), "1500 mm");
        assert_eq!(extreme(true, vec![Value::Int(3), Value::Int(9), Value::Int(1)], None).unwrap().repr(), "9");
        assert!(call(Builtin::Round, &[Value::Qty(1.0, Dim::LENGTH)]).unwrap_err().message.contains("x / mm"));
    }

    #[test]
    fn list_text_and_dictionary_methods() {
        let l = Value::list(vec![Value::Int(3), Value::Int(1)]);
        method(&l, "append", &[Value::Int(2)], &[]).unwrap();
        method(&l, "sort", &[], &[]).unwrap();
        assert_eq!(l.repr(), "[1, 2, 3]");
        assert_eq!(method(&l, "pop", &[Value::Int(0)], &[]).unwrap().repr(), "1");
        assert_eq!(method(&Value::str("a,b"), "split", &[Value::str(",")], &[]).unwrap().repr(), "['a', 'b']");
        let d = Value::dict(Dict::default());
        method(&d, "setdefault", &[Value::str("k"), Value::Int(0)], &[]).unwrap();
        assert_eq!(method(&d, "get", &[Value::str("k")], &[]).unwrap().repr(), "0");
        assert_eq!(method(&d, "get", &[Value::str("x"), Value::Int(9)], &[]).unwrap().repr(), "9");
        assert_eq!(method(&l, "fly", &[], &[]).unwrap_err().code, "unknownAttribute");
    }

    #[test]
    fn random_numbers_repeat_with_the_seed_and_keep_their_units() {
        let (mut a, mut b) = (Rng::new(1, 7), Rng::new(1, 7));
        let args = [Value::Qty(20.0, Dim::TIME), Value::Qty(40.0, Dim::TIME)];
        let x = random(Builtin::RandomUniform, &mut a, &args, &[]).unwrap();
        let y = random(Builtin::RandomUniform, &mut b, &args, &[]).unwrap();
        assert!(x.equals(&y));
        assert!(matches!(x, Value::Qty(t, Dim::TIME) if (20.0..40.0).contains(&t)));
        let mut r = Rng::new(3, 1);
        let mean: f64 = (0..10_000).map(|_| r.normal()).sum::<f64>() / 10_000.0;
        assert!(mean.abs() < 0.05);
    }
}
