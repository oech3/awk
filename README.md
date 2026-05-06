# uutils AWK

This is a human, WIP, and clean implementation of an AWK interpreter, written in Rust and compatible with GNU's AWK (`gawk`) bug-for-bug. Expected to be production-ready before Ubuntu 26.10. Made with love.

## State of the Repo

### Lexer

Pretty much done; it is lacking a POSIX-compatibility mode (trivial) and arena integration (trivial).

### Parser

Also mostly done; some bullet points:

* Rough edges around error handling on expected tokens (remove `todo!()` and friends on a few error branches; trivial).
* Extend spans during Pratt parsing for better errors (trivial-ish?).
* The preprocessor is TBD.
* It would be nice to reduce LOC.
* We are working on test coverage and fuzzing.
  * It's possible we move to the `chumsky` crate for this reason and `ariadne` support. I currently think it's not necessary, though; we probably are better suited to owning the parser given how idiosyncratic and ambiguous AWK's grammar is.
* Start running awk parsing tests at some point (especially when we get a basic interpreter and nail down `--pretty-print`).

### Interpreter

We are looking forward to building a basic tree-walking interpreter to get integration testing going, as well as a baseline for future iterations. Ideally, these should be a bytecode machine or a JIT. Expect this to be a fast-paced repo. The design sketch is for it to be a cooperative I/O machine, probably built with `smol`; if we want to better support AWK's long-forgotten number-crunching intent, we could easily extend this to parallel computations.

## Contributing

See [this](https://github.com/uutils/coreutils/blob/main/CONTRIBUTING.md).

## License

This is licensed under either the MIT License or the Apache License v2.0. See the `LICENSE-MIT` and `LICENSE-APACHE` files for details.
