# Postcard-IDL

`pidl`, for short

## Todos

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
