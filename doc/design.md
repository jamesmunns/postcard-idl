Keywords:

* Primitives
    * `bool`
    * `i8`
    * `i16`
    * `i32`
    * `i64`
    * `i128`
    * `u8`
    * `u16`
    * `u32`
    * `u64`
    * `u128`
    * `f32`
    * `f64`
    * `char`
    * `string`
    * `bytearray`
    * `unit`
* Composite
    * `option`
    * `unitstruct`
    * `newtypestruct`
    * `seq`
    * `tuple`
    * `tuplestruct`
    * `map`
    * `struct`
    * `enum`
* Enum Variants
    * `unit variant`
    * `newtype variant`
    * `tuple variant`
    * `struct variant`

I need to be able to specify:

* types
    * aliases
    * composite types
* endpoints
* topics
* "tagged types", basically a type + a name
    * Do we just call these "files"?
* protocols
    * types
    * endpoint
    * topics
* imports

Fake syntax

Shout out to https://github.com/Gankra/kdl-script

```kdl
types {
    struct "Rgb8" {
        r "u8"
    }
}
```
