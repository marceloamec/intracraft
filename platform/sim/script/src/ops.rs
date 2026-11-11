//! Arithmetic, comparisons, indexing and formatting on values, with Python's rules plus physical units: an error is a
//! [`Fault`] with a stable code (A17 §13.2) and a message that says what to write instead.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::ast::{BinOp, CmpOp, UnOp};
use crate::units::{Dim, display_parts, number_text};
use crate::value::{Value, float_text, lock};

/// Most items a list, tuple or dictionary may hold, and characters a text may hold (A17 §14.5).
pub const MAX_ITEMS: usize = 100_000;

/// A run error before it knows its block, line and time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub code: &'static str,
    pub message: String,
}

impl Fault {
    pub fn new(code: &'static str, message: impl Into<String>) -> Fault {
        Fault { code, message: message.into() }
    }

    pub fn types(message: impl Into<String>) -> Fault {
        Fault::new("typeMismatch", message)
    }
}

pub type Res<T> = Result<T, Fault>;

pub fn unary(op: UnOp, v: &Value) -> Res<Value> {
    match op {
        UnOp::Not => Ok(Value::Bool(!v.truthy())),
        UnOp::Pos => match v {
            Value::Int(_) | Value::Float(_) | Value::Qty(..) => Ok(v.clone()),
            Value::Bool(b) => Ok(Value::Int(i64::from(*b))),
            _ => Err(Fault::types(format!("`+` needs a number, not a {}", v.type_name()))),
        },
        UnOp::Neg => match v {
            Value::Int(i) => i.checked_neg().map(Value::Int).ok_or_else(overflow),
            Value::Bool(b) => Ok(Value::Int(-i64::from(*b))),
            Value::Float(x) => Ok(Value::Float(-x)),
            Value::Qty(x, d) => Ok(Value::Qty(-x, *d)),
            _ => Err(Fault::types(format!("`-` needs a number, not a {}", v.type_name()))),
        },
    }
}

fn overflow() -> Fault {
    Fault::new("tooLarge", "a whole number grew beyond ±9.2e18")
}

fn zero_division() -> Fault {
    Fault::new("divisionByZero", "division by zero")
}

pub fn binary(op: BinOp, a: &Value, b: &Value) -> Res<Value> {
    // Text, lists and tuples.
    match (op, a, b) {
        (BinOp::Add, Value::Str(x), Value::Str(y)) => {
            check_len(x.chars().count() + y.chars().count())?;
            return Ok(Value::Str(Arc::from(format!("{x}{y}"))));
        }
        (BinOp::Add, Value::List(x), Value::List(y)) => {
            let mut items = lock(x).clone();
            items.extend(lock(y).iter().cloned());
            check_len(items.len())?;
            return Ok(Value::list(items));
        }
        (BinOp::Add, Value::Tuple(x), Value::Tuple(y)) => {
            let items: Vec<Value> = x.iter().chain(y.iter()).cloned().collect();
            check_len(items.len())?;
            return Ok(Value::tuple(items));
        }
        (BinOp::Mul, Value::Str(_) | Value::List(_) | Value::Tuple(_), Value::Int(_) | Value::Bool(_)) => return repeat(a, b),
        (BinOp::Mul, Value::Int(_) | Value::Bool(_), Value::Str(_) | Value::List(_) | Value::Tuple(_)) => return repeat(b, a),
        (BinOp::Add, Value::Str(_), _) | (BinOp::Add, _, Value::Str(_)) => {
            return Err(Fault::types(format!(
                "can't add a {} and a {}; write str(x) to turn a value into text, or use an f-string",
                a.type_name(),
                b.type_name()
            )));
        }
        _ => {}
    }
    // Whole numbers stay whole where Python keeps them so.
    if let (Some(x), Some(y)) = (int_of(a), int_of(b)) {
        return int_binary(op, x, y);
    }
    let (Some((x, dx)), Some((y, dy))) = (a.as_quantity(), b.as_quantity()) else {
        return Err(Fault::types(format!("`{}` doesn't work on a {} and a {}", op.symbol(), a.type_name(), b.type_name())));
    };
    let same = |what: &str| -> Res<()> { if dx == dy { Ok(()) } else { Err(unit_mismatch(what, dx, dy)) } };
    let out = match op {
        BinOp::Add => {
            same("add")?;
            Value::qty(x + y, dx)
        }
        BinOp::Sub => {
            same("subtract")?;
            Value::qty(x - y, dx)
        }
        BinOp::Mul => Value::qty(tidy(x * y, dx, dy), dx.times(dy).ok_or_else(dim_overflow)?),
        BinOp::Div => {
            if y == 0.0 {
                return Err(zero_division());
            }
            Value::qty(tidy(x / y, dx, dy), dx.per(dy).ok_or_else(dim_overflow)?)
        }
        BinOp::FloorDiv => {
            same("divide with //")?;
            if y == 0.0 {
                return Err(zero_division());
            }
            Value::Float((x / y).floor())
        }
        BinOp::Mod => {
            same("take the remainder of")?;
            if y == 0.0 {
                return Err(zero_division());
            }
            Value::qty(x - y * (x / y).floor(), dx)
        }
        BinOp::Pow => {
            if !dy.is_none() {
                return Err(Fault::new("unitMismatch", "an exponent can't have a unit"));
            }
            if dx.is_none() {
                Value::Float(x.powf(y))
            } else {
                if y.fract() != 0.0 {
                    return Err(Fault::new("unitMismatch", format!("a {} can only be raised to a whole power", dx.name())));
                }
                Value::qty(x.powf(y), dx.pow(y as i64).ok_or_else(dim_overflow)?)
            }
        }
    };
    if let Value::Float(f) | Value::Qty(f, _) = out
        && f.is_nan()
        && !x.is_nan()
        && !y.is_nan()
    {
        return Err(Fault::new("notANumber", format!("`{}` gave no number (nan)", op.symbol())));
    }
    Ok(out)
}

/// A product or quotient that converts units, rounded to 14 significant digits: `150 * mm / mm` is 150, not
/// 149.99999999999997 (the decimal unit sizes have no exact binary value). Plain numbers are left as Python leaves them.
fn tidy(v: f64, a: Dim, b: Dim) -> f64 {
    if (a.is_none() && b.is_none()) || v == 0.0 || !v.is_finite() {
        return v;
    }
    let scale = 10f64.powi(13 - v.abs().log10().floor() as i32);
    if !scale.is_finite() || scale == 0.0 {
        return v;
    }
    let r = (v * scale).round() / scale;
    if r.is_finite() { r } else { v }
}

fn dim_overflow() -> Fault {
    Fault::new("unitMismatch", "the units of this result are too complex")
}

pub fn unit_mismatch(what: &str, a: Dim, b: Dim) -> Fault {
    let hint = if a.is_none() || b.is_none() {
        let named = if a.is_none() { b } else { a };
        format!("; give the plain number its unit, for example `2 * {}`", example_unit(named))
    } else {
        String::new()
    };
    Fault::new("unitMismatch", format!("can't {what} a {} and a {}{hint}", a.name(), b.name()))
}

fn example_unit(d: Dim) -> &'static str {
    match d {
        Dim::LENGTH => "mm",
        Dim::TIME => "s",
        Dim::MASS => "kg",
        Dim::ANGLE => "deg",
        Dim::SPEED => "m / s",
        Dim::ACCELERATION => "m / s**2",
        _ => "the unit",
    }
}

fn int_of(v: &Value) -> Option<i64> {
    match v {
        Value::Int(i) => Some(*i),
        Value::Bool(b) => Some(i64::from(*b)),
        _ => None,
    }
}

fn int_binary(op: BinOp, x: i64, y: i64) -> Res<Value> {
    Ok(match op {
        BinOp::Add => Value::Int(x.checked_add(y).ok_or_else(overflow)?),
        BinOp::Sub => Value::Int(x.checked_sub(y).ok_or_else(overflow)?),
        BinOp::Mul => Value::Int(x.checked_mul(y).ok_or_else(overflow)?),
        BinOp::Div => {
            if y == 0 {
                return Err(zero_division());
            }
            Value::Float(x as f64 / y as f64)
        }
        BinOp::FloorDiv => {
            if y == 0 {
                return Err(zero_division());
            }
            // Python's `//` rounds toward minus infinity.
            let q = x.checked_div(y).ok_or_else(overflow)?;
            Value::Int(if x % y != 0 && ((x < 0) != (y < 0)) { q - 1 } else { q })
        }
        BinOp::Mod => {
            if y == 0 {
                return Err(zero_division());
            }
            let r = x.checked_rem(y).ok_or_else(overflow)?;
            Value::Int(if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r })
        }
        BinOp::Pow => {
            if y < 0 {
                Value::Float((x as f64).powf(y as f64))
            } else {
                let e = u32::try_from(y).map_err(|_| overflow())?;
                Value::Int(x.checked_pow(e).ok_or_else(overflow)?)
            }
        }
    })
}

fn check_len(n: usize) -> Res<()> {
    if n > MAX_ITEMS { Err(Fault::new("tooLarge", format!("more than {MAX_ITEMS} items or characters"))) } else { Ok(()) }
}

fn repeat(seq: &Value, n: &Value) -> Res<Value> {
    let n = usize::try_from(int_of(n).unwrap_or(0).max(0)).unwrap_or(0);
    match seq {
        Value::Str(s) => {
            check_len(s.chars().count().saturating_mul(n))?;
            Ok(Value::Str(Arc::from(s.repeat(n))))
        }
        Value::List(l) => {
            let items = lock(l).clone();
            check_len(items.len().saturating_mul(n))?;
            Ok(Value::list(items.iter().cloned().cycle().take(items.len() * n).collect()))
        }
        Value::Tuple(t) => {
            check_len(t.len().saturating_mul(n))?;
            Ok(Value::tuple(t.iter().cloned().cycle().take(t.len() * n).collect()))
        }
        _ => Err(Fault::types("only text, lists and tuples repeat with `*`")),
    }
}

pub fn compare(op: CmpOp, a: &Value, b: &Value) -> Res<bool> {
    Ok(match op {
        CmpOp::Eq => equal_checked(a, b)?,
        CmpOp::Ne => !equal_checked(a, b)?,
        CmpOp::Lt => order(a, b)? == Ordering::Less,
        CmpOp::Le => order(a, b)? != Ordering::Greater,
        CmpOp::Gt => order(a, b)? == Ordering::Greater,
        CmpOp::Ge => order(a, b)? != Ordering::Less,
        CmpOp::In => contains(b, a)?,
        CmpOp::NotIn => !contains(b, a)?,
        CmpOp::Is => same(a, b),
        CmpOp::IsNot => !same(a, b),
    })
}

/// `==`, refusing to compare quantities of different dimensions (always a mistake, never just false).
fn equal_checked(a: &Value, b: &Value) -> Res<bool> {
    if let (Some((_, da)), Some((_, db))) = (a.as_quantity(), b.as_quantity())
        && da != db
    {
        return Err(unit_mismatch("compare", da, db));
    }
    Ok(a.equals(b))
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::List(x), Value::List(y)) => Arc::ptr_eq(x, y),
        (Value::Dict(x), Value::Dict(y)) => Arc::ptr_eq(x, y),
        (Value::None, Value::None) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Object(x), Value::Object(y)) => x == y,
        (Value::Pallet(x), Value::Pallet(y)) => x == y,
        (Value::None, _) | (_, Value::None) => false,
        _ => a.equals(b),
    }
}

/// Python's ordering: numbers (of one dimension), texts, and lists or tuples item by item.
pub fn order(a: &Value, b: &Value) -> Res<Ordering> {
    match (a, b) {
        (Value::Str(x), Value::Str(y)) => Ok(x.cmp(y)),
        (Value::List(x), Value::List(y)) => {
            let (xs, ys) = (lock(x).clone(), lock(y).clone());
            order_seq(&xs, &ys)
        }
        (Value::Tuple(x), Value::Tuple(y)) => order_seq(x, y),
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        _ => match (a.as_quantity(), b.as_quantity()) {
            (Some((x, dx)), Some((y, dy))) => {
                if dx != dy {
                    return Err(unit_mismatch("compare", dx, dy));
                }
                x.partial_cmp(&y).ok_or_else(|| Fault::new("notANumber", "nan can't be compared"))
            }
            _ => Err(Fault::types(format!("can't order a {} and a {}", a.type_name(), b.type_name()))),
        },
    }
}

fn order_seq(xs: &[Value], ys: &[Value]) -> Res<Ordering> {
    for (x, y) in xs.iter().zip(ys) {
        if !x.equals(y) {
            return order(x, y);
        }
    }
    Ok(xs.len().cmp(&ys.len()))
}

/// `item in container`.
pub fn contains(container: &Value, item: &Value) -> Res<bool> {
    match container {
        Value::Str(s) => match item {
            Value::Str(t) => Ok(s.contains(t.as_ref())),
            _ => Err(Fault::types("`in` a text needs a text on its left")),
        },
        Value::List(l) => Ok(lock(l).clone().iter().any(|v| v.equals(item))),
        Value::Tuple(t) => Ok(t.iter().any(|v| v.equals(item))),
        Value::Dict(d) => lock(d).contains(item).map_err(Fault::types),
        Value::Range(a, b, s) => {
            let Some(i) = int_of(item) else { return Ok(false) };
            Ok(if *s > 0 { i >= *a && i < *b && (i - a) % s == 0 } else { i <= *a && i > *b && (a - i) % (-s) == 0 })
        }
        _ => Err(Fault::types(format!("`in` doesn't look inside a {}", container.type_name()))),
    }
}

/// `seq[index]` and `seq[a:b:c]`.
pub fn get_item(base: &Value, index: &Value) -> Res<Value> {
    if let Value::Slice(a, b, c) = index {
        return slice(base, *a, *b, *c);
    }
    match base {
        Value::List(l) => {
            let items = lock(l);
            let i = position(index, items.len())?;
            Ok(items[i].clone())
        }
        Value::Tuple(t) => {
            let i = position(index, t.len())?;
            Ok(t[i].clone())
        }
        Value::Str(s) => {
            let chars: Vec<char> = s.chars().collect();
            let i = position(index, chars.len())?;
            Ok(Value::str(&chars[i].to_string()))
        }
        Value::Range(a, b, s) => {
            let len = range_len(*a, *b, *s);
            let i = position(index, len)?;
            Ok(Value::Int(a + s * i as i64))
        }
        Value::Dict(d) => {
            let found = lock(d).get(index).map_err(Fault::types)?;
            found.ok_or_else(|| Fault::new("keyNotFound", format!("the dictionary has no key {}", index.repr())))
        }
        _ => Err(Fault::types(format!("a {} has no items to index", base.type_name()))),
    }
}

pub fn set_item(base: &Value, index: &Value, value: Value) -> Res<()> {
    match base {
        Value::List(l) => {
            let mut items = lock(l);
            let i = position(index, items.len())?;
            items[i] = value;
            Ok(())
        }
        Value::Dict(d) => {
            let mut dict = lock(d);
            if dict.len() >= MAX_ITEMS && !dict.contains(index).unwrap_or(false) {
                return Err(Fault::new("tooLarge", format!("a dictionary holds at most {MAX_ITEMS} keys")));
            }
            dict.set(index.clone(), value).map_err(Fault::types)
        }
        Value::Tuple(_) | Value::Str(_) => Err(Fault::types(format!("a {} can't be changed; make a new one", base.type_name()))),
        _ => Err(Fault::types(format!("a {} has no items to set", base.type_name()))),
    }
}

/// An index counted from the start, or from the end when negative; checked against the length.
pub fn position(index: &Value, len: usize) -> Res<usize> {
    let Some(i) = int_of(index) else {
        return Err(Fault::types(format!("an index is a whole number, not a {}", index.type_name())));
    };
    let n = i64::try_from(len).unwrap_or(i64::MAX);
    let j = if i < 0 { i + n } else { i };
    if j < 0 || j >= n {
        return Err(Fault::new("indexOutOfRange", format!("index {i} is outside 0 … {} (it has {len} items)", n - 1)));
    }
    Ok(j as usize)
}

pub fn range_len(a: i64, b: i64, s: i64) -> usize {
    if s > 0 && b > a {
        usize::try_from((b - a + s - 1) / s).unwrap_or(0)
    } else if s < 0 && a > b {
        usize::try_from((a - b - s - 1) / (-s)).unwrap_or(0)
    } else {
        0
    }
}

fn slice(base: &Value, a: Option<i64>, b: Option<i64>, c: Option<i64>) -> Res<Value> {
    let step = c.unwrap_or(1);
    if step == 0 {
        return Err(Fault::new("indexOutOfRange", "a slice's step can't be 0"));
    }
    let pick = |len: usize| -> Vec<usize> {
        let n = i64::try_from(len).unwrap_or(i64::MAX);
        let clamp = |v: i64, lo: i64, hi: i64| v.max(lo).min(hi);
        let norm = |v: i64| if v < 0 { v + n } else { v };
        let mut out = Vec::new();
        if step > 0 {
            let (mut i, end) = (clamp(norm(a.unwrap_or(0)), 0, n), clamp(norm(b.unwrap_or(n)), 0, n));
            while i < end {
                out.push(i as usize);
                i += step;
            }
        } else {
            let (mut i, end) = (clamp(a.map_or(n - 1, norm), -1, n - 1), b.map_or(-1, |v| clamp(norm(v), -1, n - 1)));
            while i > end {
                out.push(i as usize);
                i += step;
            }
        }
        out
    };
    match base {
        Value::List(l) => {
            let items = lock(l).clone();
            Ok(Value::list(pick(items.len()).into_iter().map(|i| items[i].clone()).collect()))
        }
        Value::Tuple(t) => Ok(Value::tuple(pick(t.len()).into_iter().map(|i| t[i].clone()).collect())),
        Value::Str(s) => {
            let chars: Vec<char> = s.chars().collect();
            Ok(Value::str(&pick(chars.len()).into_iter().map(|i| chars[i]).collect::<String>()))
        }
        _ => Err(Fault::types(format!("a {} can't be sliced", base.type_name()))),
    }
}

/// An f-string's `{value:spec}`: `[[fill]align][0][width][.precision][type]`, type one of `d f e g % s`. A quantity is
/// formatted as its number in IntraCraft's display unit, followed by the unit.
pub fn format_value(v: &Value, spec: &str) -> Res<String> {
    if spec.is_empty() {
        return Ok(v.to_text());
    }
    let chars: Vec<char> = spec.chars().collect();
    let mut i = 0;
    let (mut fill, mut align) = (' ', None);
    if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^') {
        fill = chars[0];
        align = Some(chars[1]);
        i = 2;
    } else if !chars.is_empty() && matches!(chars[0], '<' | '>' | '^') {
        align = Some(chars[0]);
        i = 1;
    }
    let mut zero = false;
    if chars.get(i) == Some(&'0') {
        zero = true;
        i += 1;
    }
    let mut width = 0usize;
    while let Some(d) = chars.get(i).and_then(|c| c.to_digit(10)) {
        width = (width * 10 + d as usize).min(1000);
        i += 1;
    }
    let mut precision = None;
    if chars.get(i) == Some(&'.') {
        i += 1;
        let mut p = 0usize;
        while let Some(d) = chars.get(i).and_then(|c| c.to_digit(10)) {
            p = (p * 10 + d as usize).min(50);
            i += 1;
        }
        precision = Some(p);
    }
    let kind = chars.get(i).copied();
    if i + usize::from(kind.is_some()) != chars.len() {
        return Err(Fault::types(format!("`{spec}` is not a format IntraScript knows (try `.2f`, `>8`, `d`)")));
    }
    let (number, unit) = match v {
        Value::Qty(x, d) => {
            let (n, u) = display_parts(*x, *d);
            (Some(n), Some(u))
        }
        other => (other.as_f64(), None),
    };
    let body = match (kind, number) {
        (Some('d'), Some(x)) => format!("{}", x.round_ties_even() as i64),
        (Some('f'), Some(x)) => format!("{x:.*}", precision.unwrap_or(6)),
        (Some('e'), Some(x)) => format!("{x:.*e}", precision.unwrap_or(6)),
        (Some('%'), Some(x)) => format!("{:.*}%", precision.unwrap_or(6), x * 100.0),
        (Some('g') | None, Some(x)) if precision.is_some() => format!("{x:.*}", precision.unwrap_or(6)),
        (None, Some(x)) if unit.is_some() => number_text(x),
        (Some('g'), Some(x)) => number_text(x),
        (None | Some('s'), _) => {
            let t = v.to_text();
            match precision {
                Some(p) => t.chars().take(p).collect(),
                None => t,
            }
        }
        (Some(k), None) => return Err(Fault::types(format!("format `{k}` needs a number, not a {}", v.type_name()))),
        (Some(k), Some(_)) => return Err(Fault::types(format!("format type `{k}` is not one of d, f, e, g, %, s"))),
    };
    let body = match unit {
        Some(u) if kind != Some('s') => format!("{body} {u}"),
        _ => body,
    };
    let len = body.chars().count();
    if len >= width {
        return Ok(body);
    }
    let pad = width - len;
    let numeric = number.is_some() && kind != Some('s');
    Ok(match (align, zero && numeric) {
        (None, true) => {
            let (sign, rest) = if let Some(r) = body.strip_prefix('-') { ("-", r) } else { ("", body.as_str()) };
            format!("{sign}{}{rest}", "0".repeat(pad))
        }
        (Some('<'), _) => format!("{body}{}", fill.to_string().repeat(pad)),
        (Some('^'), _) => {
            let left = pad / 2;
            format!("{}{body}{}", fill.to_string().repeat(left), fill.to_string().repeat(pad - left))
        }
        (None, false) if !numeric => format!("{body}{}", " ".repeat(pad)),
        _ => format!("{}{body}", fill.to_string().repeat(pad)),
    })
}

/// A float as `str()` shows it.
pub fn float_str(x: f64) -> String {
    float_text(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(x: f64, d: Dim) -> Value {
        Value::Qty(x, d)
    }

    #[test]
    fn whole_numbers_follow_python() {
        assert!(matches!(binary(BinOp::Div, &Value::Int(7), &Value::Int(2)).unwrap(), Value::Float(x) if x == 3.5));
        assert!(matches!(binary(BinOp::FloorDiv, &Value::Int(-7), &Value::Int(2)).unwrap(), Value::Int(-4)));
        assert!(matches!(binary(BinOp::Mod, &Value::Int(-7), &Value::Int(2)).unwrap(), Value::Int(1)));
        assert!(matches!(binary(BinOp::Pow, &Value::Int(2), &Value::Int(10)).unwrap(), Value::Int(1024)));
        assert_eq!(binary(BinOp::Div, &Value::Int(1), &Value::Int(0)).unwrap_err().code, "divisionByZero");
        assert_eq!(binary(BinOp::Mul, &Value::Int(i64::MAX), &Value::Int(2)).unwrap_err().code, "tooLarge");
    }

    #[test]
    fn units_combine_and_mismatches_are_errors() {
        let gap = q(0.15, Dim::LENGTH);
        let v = q(0.25, Dim::SPEED);
        let t = binary(BinOp::Div, &gap, &v).unwrap();
        assert!(matches!(t, Value::Qty(x, Dim::TIME) if (x - 0.6).abs() < 1e-12));
        let n = binary(BinOp::Div, &gap, &q(0.001, Dim::LENGTH)).unwrap();
        assert!(matches!(n, Value::Float(x) if (x - 150.0).abs() < 1e-9));
        let e = binary(BinOp::Add, &gap, &q(2.0, Dim::TIME)).unwrap_err();
        assert_eq!(e.code, "unitMismatch");
        assert!(binary(BinOp::Add, &gap, &Value::Int(2)).unwrap_err().message.contains("2 * mm"));
        assert!(compare(CmpOp::Lt, &gap, &q(1.0, Dim::TIME)).is_err());
        assert!(compare(CmpOp::Lt, &gap, &q(1.0, Dim::LENGTH)).unwrap());
    }

    #[test]
    fn indexes_and_slices_count_from_zero_and_from_the_end() {
        let l = Value::list((0..5).map(Value::Int).collect());
        assert!(matches!(get_item(&l, &Value::Int(-1)).unwrap(), Value::Int(4)));
        assert_eq!(get_item(&l, &Value::Int(5)).unwrap_err().code, "indexOutOfRange");
        assert_eq!(get_item(&l, &Value::Slice(Some(1), Some(3), None)).unwrap().repr(), "[1, 2]");
        assert_eq!(get_item(&l, &Value::Slice(None, None, Some(-2))).unwrap().repr(), "[4, 2, 0]");
        assert_eq!(get_item(&Value::str("pallet"), &Value::Slice(None, Some(3), None)).unwrap().to_text(), "pal");
    }

    #[test]
    fn formats_numbers_and_quantities() {
        assert_eq!(format_value(&Value::Float(1.23456), ".2f").unwrap(), "1.23");
        assert_eq!(format_value(&Value::Int(7), "03d").unwrap(), "007");
        assert_eq!(format_value(&Value::str("ab"), ">4").unwrap(), "  ab");
        assert_eq!(format_value(&q(0.15, Dim::LENGTH), ".1f").unwrap(), "150.0 mm");
        assert_eq!(format_value(&Value::Float(0.256), ".1%").unwrap(), "25.6%");
        assert!(format_value(&Value::Int(1), "q").is_err());
    }
}
