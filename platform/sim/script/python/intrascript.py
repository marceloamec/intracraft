"""IntraScript's names for Python editors (A17 IntraScript Language, §17).

IntraScript code is written with Python's syntax, so any Python editor colours it. Put this file next to your code (or
on PYTHONPATH) and write ``from intrascript import *`` at the top of a code block: the editor then knows every event,
function and unit, completes them and shows their help. IntraCraft itself ignores the import.

This module only describes the names. It does not simulate anything: run the code in IntraCraft (the Discrete
simulation module, or ``intracraft-cli code run LAYOUT CODE.py``). Plain Python can import a block's code with it, so a
syntax check or a linter works, but ``wait()`` returns at once and the objects of the layout are unknown to Python.

License: MIT OR Apache-2.0, like IntraCraft.
"""

from __future__ import annotations

import math as _math
import random as _random
from typing import Any, Callable, Iterable, Optional, TypeVar

__all__ = [
    "on_start", "on_finish", "on_create", "on_enter", "on_leave", "on_rise", "on_fall", "on_change", "every", "at",
    "choose_exit", "wait", "wait_until", "now", "warn", "error", "math", "random", "simulation",
    "mm", "cm", "m", "km", "ms", "s", "minute", "h", "day", "g", "kg", "tonne", "deg", "rad", "rpm", "percent",
]

F = TypeVar("F", bound=Callable[..., Any])

# ---- units: sizes in SI, as IntraScript keeps quantities (in IntraScript they also carry their dimension) ----
mm, cm, m, km = 0.001, 0.01, 1.0, 1000.0
ms, s, minute, h, day = 0.001, 1.0, 60.0, 3600.0, 86400.0
g, kg, tonne = 0.001, 1.0, 1000.0
deg, rad, rpm = _math.pi / 180.0, 1.0, 2.0 * _math.pi / 60.0
percent = 0.01


# ---- events (decorators) ----
def _event(*_objects: Any, priority: int = 0, **_options: Any) -> Callable[[F], F]:
    def bind(f: F) -> F:
        return f

    return bind


def on_start(f: F) -> F:
    """When the run starts, after the block's variables have their values."""
    return f


def on_finish(f: F) -> F:
    """When the run ends. It can't wait."""
    return f


def on_create(*sources: Any, priority: int = 0) -> Callable[[F], F]:
    """A source made a pallet. The handler may take `pallet` and `sender`."""
    return _event(*sources, priority=priority)


def on_enter(*objects: Any, priority: int = 0) -> Callable[[F], F]:
    """A pallet now belongs to the object (in 2D: its tail has left the object before)."""
    return _event(*objects, priority=priority)


def on_leave(*objects: Any, priority: int = 0) -> Callable[[F], F]:
    """A pallet no longer belongs to the object."""
    return _event(*objects, priority=priority)


def on_rise(*signals: Any, priority: int = 0) -> Callable[[F], F]:
    """A signal turns True: `@on_rise(Photoeye3.detected)`."""
    return _event(*signals, priority=priority)


def on_fall(*signals: Any, priority: int = 0) -> Callable[[F], F]:
    """A signal turns False."""
    return _event(*signals, priority=priority)


def on_change(*signals: Any, priority: int = 0) -> Callable[[F], F]:
    """A signal changes, either way."""
    return _event(*signals, priority=priority)


def every(interval: float, start: Optional[float] = None, priority: int = 0) -> Callable[[F], F]:
    """Again and again: `@every(10 * minute)`; the first time at `start` (default: after one interval)."""
    return _event(interval, priority=priority)


def at(time: float, priority: int = 0) -> Callable[[F], F]:
    """Once, at that time: `@at(2 * h)`."""
    return _event(time, priority=priority)


def choose_exit(*objects: Any, priority: int = 0) -> Callable[[F], F]:
    """Where a pallet leaves a turntable: return one of its exits, or None for its own exit rule. It can't wait."""
    return _event(*objects, priority=priority)


# ---- waiting and time ----
def wait(duration: float) -> None:
    """Pauses this handler for a time of the simulation (`wait(2 * s)`); the plant goes on meanwhile."""


def wait_until(condition: Any, timeout: Optional[float] = None) -> bool:
    """Pauses until the condition holds (looked at again after every event); False when `timeout` passed first.

    In IntraScript the condition is evaluated again and again; in plain Python it is evaluated once.
    """
    return bool(condition)


def now() -> float:
    """The simulation time (a time quantity in IntraScript)."""
    return 0.0


def warn(*values: Any) -> None:
    """Like print(), as a warning of the run."""
    print("warning:", *values)


def error(*values: Any) -> None:
    """Stops the run with this message."""
    raise RuntimeError(" ".join(str(v) for v in values))


# ---- modules ----
class _Math:
    pi, e, inf = _math.pi, _math.e, _math.inf
    sqrt, sin, cos, tan, atan2 = _math.sqrt, _math.sin, _math.cos, _math.tan, _math.atan2
    floor, ceil, hypot, isclose, exp, log = _math.floor, _math.ceil, _math.hypot, _math.isclose, _math.exp, _math.log


class _Random:
    """The block's own seeded stream: the same seed gives the same numbers."""

    random = staticmethod(_random.random)
    uniform = staticmethod(_random.uniform)
    randint = staticmethod(_random.randint)
    triangular = staticmethod(_random.triangular)

    @staticmethod
    def normal(mean: float, std_dev: float) -> float:
        return _random.gauss(mean, std_dev)

    gauss = normal

    @staticmethod
    def exponential(mean: float) -> float:
        """Exponential with this MEAN (Python's expovariate takes the rate)."""
        return _random.expovariate(1.0 / mean)

    @staticmethod
    def chance(p: float) -> bool:
        """True with probability p."""
        return _random.random() < p

    @staticmethod
    def choice(items: Iterable[Any]) -> Any:
        return _random.choice(list(items))


class _Simulation:
    mode = "2D"
    seed = 1

    @staticmethod
    def stop(reason: str = "") -> None:
        """Ends the run at the end of this instant."""


math = _Math()
random = _Random()
simulation = _Simulation()
