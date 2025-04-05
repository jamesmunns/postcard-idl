Keywords:

* Primitives
    * `bool` x
    * `i8` x
    * `i16` x
    * `i32` x
    * `i64` x
    * `i128` x
    * `u8` x
    * `u16` x
    * `u32` x
    * `u64` x
    * `u128` x
    * `f32` x
    * `f64` x
    * `char` x
    * `string` x
    * `bytearray` is this just `[u8]`?
    * `unit` x
* Composite
    * `struct` x
    * `unitstruct` x
    * `newtypestruct` x
    * `tuplestruct` x
    * `option` x
    * `seq` x
    * `tuple` x
    * `map` x
    * `enum` x
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

    struct "UnitStruct"

    struct "NewTypeStruct" "bool"

    struct "TupleStruct" {
        _0 "i32"
        _1 "i32"
    }

    enum "Example" {
        UnitVariant
        NewtypeVariant "Rgb8"
        TupleVariant {
            _0 "u32"
        }
        StructVariant {
            bar "u64"
        }
    }

    struct "Primitives" {
        "ex_bool"   "bool"
        "ex_i8"     "i8"
        "ex_i16"    "i16"
        "ex_i32"    "i32"
        "ex_i64"    "i64"
        "ex_i128"   "i128"
        "ex_u8"     "u8"
        "ex_u16"    "u16"
        "ex_u32"    "u32"
        "ex_u64"    "u64"
        "ex_u128"   "u128"
        "ex_f32"    "f32"
        "ex_f64"    "f64"
        "ex_char"   "char"
        "ex_string" "string"
        "ex_unit"   "unit"
    }

    // All the types here can decl in usage position
    struct "AdHocTypes" {
        options "option<u64>"
        seqs "[u8]"
        tuples "(bool, bool, u8)"
        arrays "[u16; 4]"
        maps "map<string, string>"
    }
}
```
