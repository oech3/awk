# uutils AWK

This is a human, WIP, and clean implementation of an AWK interpreter, written in Rust and compatible with GNU's AWK (`gawk`) bug-for-bug. Expected to be production-ready before Ubuntu 26.10. Made with love.

## State of the Repo

### Lexer

Pretty much done; lacking a POSIX-compatibility mode (trivial) and arena integration (trivial).

### Parser

Also mostly done; some bullet points:

* Rough edges around error handling on expected tokens.
* Preprocessor is TBD.
* Missing command redirections.
* We are working on test coverage and fuzzing.
* It would be nice to reduce LOC; maybe we get rid of S-expr debug output.
  * It's possible we move to the `chumsky` crate for this reason and Ariadne support. I currently think it's not necessary tho.

### Interpreter

TBD; work on it will be started once we get good testing on the parser. Ideally, the design should be a bytecode machine or a JIT. Expect this to be a fast-paced repo.

## Contributing

See [this](https://github.com/uutils/coreutils/blob/main/CONTRIBUTING.md).

## License

This is licensed under either the MIT License or the Apache License v2.0. See the `LICENSE-MIT` and `LICENSE-APACHE` files for details.
