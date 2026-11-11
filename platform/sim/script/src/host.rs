//! What code sees of the model, and the engine that carries out what code asks. The language knows nothing of
//! conveyors: an engine (the 2D run, later the 3D run) describes its objects with [`ModelInfo`] for the checker and
//! answers the reads, writes and calls of a running program through [`Host`].

use crate::ops::Fault;
use crate::value::Value;

/// The objects code can name, in layout order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelInfo {
    pub objects: Vec<ObjectInfo>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectInfo {
    /// The name code uses: an identifier (`Roller3`).
    pub name: String,
    /// The object's id in the layout.
    pub id: String,
    /// Its library definition (`conveyor.roller`, `info.code`).
    pub kind: String,
    pub members: Vec<Member>,
    /// Pallets go through it: `@on_enter` and `@on_leave` apply.
    pub carries_pallets: bool,
    /// It makes pallets: `@on_create` applies.
    pub creates_pallets: bool,
    /// It asks where a pallet goes: `@choose_exit` applies.
    pub decides_exit: bool,
    /// Where the object is in the layout, as code writes it: `Seccion1.Conveyor1`, or its name alone (ADR-0029).
    pub address: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub name: String,
    pub kind: MemberKind,
    /// One line for the editor's list of names.
    pub help: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberKind {
    /// A boolean signal; inputs are writable (`run`, `enable`), outputs are not (`occupied`, `detected`).
    Signal { writable: bool },
    /// A value code reads (a parameter, `count`, `pallets`, `stats`…).
    Value,
    /// Something code calls.
    Method,
}

impl ModelInfo {
    /// The object code calls `name`.
    pub fn find(&self, name: &str) -> Option<u32> {
        self.objects.iter().position(|o| o.name == name).and_then(|i| u32::try_from(i).ok())
    }

    /// The object named `name` or with the id `name` (an id is how the layout knows it).
    pub fn find_name_or_id(&self, name: &str) -> Option<u32> {
        self.find(name).or_else(|| self.objects.iter().position(|o| o.id == name).and_then(|i| u32::try_from(i).ok()))
    }

    pub fn object(&self, i: u32) -> Option<&ObjectInfo> {
        self.objects.get(i as usize)
    }
}

impl ObjectInfo {
    pub fn member(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    pub fn is_code_block(&self) -> bool {
        self.kind == crate::CODE_BLOCK
    }
}

/// The engine a program runs in. Every method answers with a value or a [`Fault`] (a stable code and a message); none
/// may panic.
pub trait Host {
    /// The simulation time, µs.
    fn now(&self) -> i64;
    /// `"2D"` or `"3D"`.
    fn mode(&self) -> &'static str;
    fn seed(&self) -> u64;
    /// A member of an object: a signal, a parameter, `count`, `pallets`, `stats`…
    fn get(&mut self, object: u32, member: &str) -> Result<Value, Fault>;
    /// Writes a member (an input signal).
    fn set(&mut self, object: u32, member: &str, value: &Value) -> Result<(), Fault>;
    /// Calls a method of an object.
    fn call(&mut self, object: u32, method: &str, args: &[Value]) -> Result<Value, Fault>;
    /// A built-in member of a pallet (`id`, `type`, `length`, `location`…); `None` when the name isn't one (it may then be
    /// an attribute code gave the pallet).
    fn pallet(&mut self, pallet: u64, member: &str) -> Result<Option<Value>, Fault>;
    /// `simulation.stop()`: the run ends at the end of this instant.
    fn stop(&mut self, reason: &str);
}

/// The built-in members of a pallet; code can't write them.
pub const PALLET_MEMBERS: [(&str, &str); 13] = [
    ("id", "Unique number in the run, in creation order"),
    ("name", "The load unit's name: MU and its number, as the address ends (ADR-0029)"),
    ("address", "Where it is: its carrier's address and its name, `Seccion1.Conveyor1.MU3`"),
    ("type", "Pallet type id (EUR, IND…)"),
    ("length", "Length along travel"),
    ("width", "Width across travel"),
    ("height", "Total height, load included"),
    ("load_mass", "Mass of the load"),
    ("created_at", "When the source made it"),
    ("age", "Time since it was made"),
    ("location", "The object carrying it now, or None"),
    ("speed", "Its speed along travel now"),
    ("attributes", "A copy of the attributes code gave it, as a dictionary"),
];
