# Postcard-IDL

`pidl`, for short

## Todos

PRs welcome on any item on these lists. Please open an issue if you want to let
me know you plan to work on any of these.

### Robustness

- [ ] Better Error Handling
    - We have spans from KDL, but we don't really use them
    - I should probably make some of "SpanStr" so errors can follow
- [ ] Any kind of testing
    - I basically just have an example that reads a single file

### Features

- [ ] Any kind of codegen
    - I should be able to generate types for Rust, and then probably other languages
- [ ] A CLI for doing... things, once we can actually do things
- [ ] Parsers for postcard-rpc features
    - Endpoints
    - Types
    - "protocols"
- [ ] Update to use the unreleased `postcard-schema` changes
- [ ] Formatter for input files - `pidl fmt`
- [ ] Imports/includes
    - [ ] Importing syntax, `use "../example.kdl" as example`?
    - [ ] referencing scoped types, e.g. `alias "Boop" "example::Booper"`
- [ ] Import types from Rust crates?
    - This would be very silly and very cool
    - `use "chrono@0.4.40`
    - doubly so if we could automagically derive `Schema` for this?
- [ ] Handle doc comments?
- [ ] Generate "Borrowed" variants, both for types themselves, AND for any types that include them
- [ ] Generate heapless types in no-std mode
    - Generic over length?

### Known Defects

- [ ] You can't define a tuple struct and then refer to it
    - define: `struct "TupleStruct" "(i32, i32)"`
    - usage: `alias "Renamed" "TupleStruct"` (doesn't work)
