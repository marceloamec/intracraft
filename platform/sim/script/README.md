# intracraft-script: IntraScript

IntraScript is IntraCraft's own programming language for simulation: Python's syntax, with events as decorators
(`@on_enter(Roller3)`, `@every(10 * minute)`), waits inside a sequence (`wait(2 * s)`, `wait_until(…, timeout=…)`),
physical units as values (`150 * mm`, `0.25 * m / s`) and the layout's objects by name.

```python
@on_rise(Photoeye3.detected)
def hold():
    Roller3.run = False
    wait(2 * s)
    Roller3.run = True
```

This crate is the whole language: lexer, parser, checker, compiler and a runtime of suspendable activations, in safe Rust
with no dependencies and no `eval`. It knows nothing of conveyors; an engine describes its objects (`host::ModelInfo`)
and answers the program (`host::Host`). The Discrete simulation module's side is `intracraft-des`
(`flow/sim/script.rs`).

- Reference: `docs/en/app/A17 IntraScript Language.md`; tutorial: `docs/en/app/A20 IntraScript Tutorial.md`.
- Decisions: ADR-0018 (the language and its objects), ADR-0027 (Python syntax, own interpreter).
- Python editors: `python/intrascript.py` describes every name for completion and help.
- Tests: `cargo test -p intracraft-script` (the language, and every example of A17, A20 and S21).

## Licence

Open source, like the rest of IntraCraft's Rust platform: MIT OR Apache-2.0, at your option. The language definition (A17)
may be implemented by anyone.
