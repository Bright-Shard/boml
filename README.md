# BOML

A dependency-free, (almost) zero-copy TOML parser for Rust.

This crate is WIP. The current goal is to be able to parse `Cargo.toml` files,
which is almost complete (see the todo below).

# Status/To-Do

The time types (date, time, date-time) aren't of importance to BOML since the
current goal is just to parse `Cargo.toml` files. They will be supported at
some point in the future, but are not right now, hence why it's marked `(future)`.

- [x] Keys
  - [x] Bare keys
  - [x] Quoted keys
  - [x] Dotted keys
- [ ] Values
  - [x] String
    - [x] Basic string
    - [x] Basic multiline string
    - [x] Literal string
    - [x] Literal multiline string
  - [x] Integer
  - [x] Float
  - [x] Boolean
  - [ ] Time (future)
    - [ ] Local Date-Time
    - [ ] Local Date
    - [ ] Local Time
  - [x] Array
- [ ] Tables
  - [x] Table
  - [x] Inline Table
  - [ ] Array of Tables
  - [x] Array of Inline Tables

# Why "(almost) zero-copy"?

TOML has 2 kinds of strings: basic strings, and literal strings. Literal strings are
just strings BOML can read from the file, but basic strings can have escapes (`\n`,
for example, gets replaced with the newline character). Processing these escapes requires
copying the string, and then replacing the escapes with their actual characters.

BOML will only copy and format a string if the string is a basic string (surrounded by `"`)
*and* actually contains escapes. Literal strings (surrounded by `'`) and basic strings without
escapes are not copied.

# Whatsitstandfor

Yes.