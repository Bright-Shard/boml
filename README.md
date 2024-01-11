# BOML

A dependency-free, no-copy TOML parser for Rust. In an absolute Rust moment,
TOML is Rust's main configuration format, and also appears to have zero serde-free
TOML parsers. BOML solves it.

This crate is WIP.

# Status/To-Do

Any features marked `(future)` are planned for the future, but not of immediate
importance. This crate is currently focused on having just enough features to
parse `Cargo.toml` and other similar files.

- [ ] Keys
  - [x] Bare keys
  - [x] Quoted keys
  - [ ] Dotted keys
- [ ] Values
  - [ ] String
    - [ ] Basic string
    - [ ] Basic multiline string
    - [x] Literal string
    - [x] Literal multiline string
  - [x] Integer
  - [x] Float
  - [x] Boolean
  - [ ] Time (future)
    - [ ] Local Date-Time
    - [ ] Local Date
    - [ ] Local Time
  - [ ] Array
- [ ] Tables
  - [ ] Table
  - [ ] Inline Table
  - [ ] Array of Tables
  - [ ] Array of Inline Tables (future)

# Whatsitstandfor

Yes.