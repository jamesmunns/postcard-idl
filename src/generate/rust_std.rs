// TODO: Probably turn most? of these methods into some kind of trait
// shared with multiple generators

use postcard_schema::schema::owned::{OwnedDataModelType, OwnedDataModelVariant, OwnedNamedType, OwnedNamedValue, OwnedNamedVariant};

use crate::Pidl;
use core::fmt::Write;

#[derive(Default, Debug)]
pub struct Output {
    pub aliases: String,
    pub types: String,
}

pub fn generate_rust_std(p: &Pidl) -> Output {
    let mut out = Output::default();
    for t in p.types.iter() {
        generate_std_ty(&mut out, t);
    }
    out
}

fn generate_std_ty(out: &mut Output, ty: &OwnedNamedType) {
    match &ty.ty {
        //
        // TODO: make sure these are aliases
        //
        OwnedDataModelType::Bool => generate_alias(out, &ty.name, "bool"),
        OwnedDataModelType::I8 => generate_alias(out, &ty.name, "i8"),
        OwnedDataModelType::U8 => generate_alias(out, &ty.name, "u8"),
        OwnedDataModelType::I16 => generate_alias(out, &ty.name, "i16"),
        OwnedDataModelType::I32 => generate_alias(out, &ty.name, "i32"),
        OwnedDataModelType::I64 => generate_alias(out, &ty.name, "i64"),
        OwnedDataModelType::I128 => generate_alias(out, &ty.name, "i128"),
        OwnedDataModelType::U16 => generate_alias(out, &ty.name, "u16"),
        OwnedDataModelType::U32 => generate_alias(out, &ty.name, "u32"),
        OwnedDataModelType::U64 => generate_alias(out, &ty.name, "u64"),
        OwnedDataModelType::U128 => generate_alias(out, &ty.name, "u128"),
        OwnedDataModelType::Usize => generate_alias(out, &ty.name, "usize"),
        OwnedDataModelType::Isize => generate_alias(out, &ty.name, "isize"),
        OwnedDataModelType::F32 => generate_alias(out, &ty.name, "f32"),
        OwnedDataModelType::F64 => generate_alias(out, &ty.name, "f64"),
        OwnedDataModelType::Char => generate_alias(out, &ty.name, "char"),
        OwnedDataModelType::String => generate_alias(out, &ty.name, "String"),
        OwnedDataModelType::ByteArray => generate_alias(out, &ty.name, "Vec<u8>"),
        OwnedDataModelType::Unit => generate_alias(out, &ty.name, "()"),

        OwnedDataModelType::Option(owned_named_type) => generate_option_alias(out, &ty.name, owned_named_type),
        OwnedDataModelType::UnitStruct => generate_unit_struct(out, &ty.name),
        OwnedDataModelType::NewtypeStruct(owned_named_type) => generate_newtype_struct(out, &ty.name, owned_named_type),
        OwnedDataModelType::Seq(owned_named_type) => generate_seq_alias(out, &ty.name, owned_named_type),
        OwnedDataModelType::Tuple(owned_named_types) => generate_tuple_alias(out, &ty.name, owned_named_types),
        OwnedDataModelType::TupleStruct(owned_named_types) => generate_tuple_struct(out, &ty.name, owned_named_types),
        OwnedDataModelType::Map { key, val } => generate_map_alias(out, &ty.name, key, val),
        OwnedDataModelType::Struct(owned_named_values) => generate_struct(out, &ty.name, owned_named_values),
        OwnedDataModelType::Enum(owned_named_variants) => generate_enum(out, &ty.name, owned_named_variants),
        OwnedDataModelType::Schema => generate_schema_alias(out, &ty.name),
    }
}

fn generate_alias(out: &mut Output, name: &str, ty: &str) {
    writeln!(&mut out.aliases, "pub type {} = {};", name, ty).unwrap();
}

fn generate_option_alias(out: &mut Output, name: &str, ont: &OwnedNamedType) {
    write!(&mut out.aliases, "pub type {} = Option<", name).unwrap();
    write_ty_refr(&mut out.aliases, ont);
    writeln!(&mut out.aliases, ">;").unwrap();
}

fn generate_tuple_alias(out: &mut Output, name: &str, owned_named_types: &[OwnedNamedType]) {
    write!(&mut out.aliases, "pub type {} = ", name).unwrap();
    write!(&mut out.aliases, "{}", tuple_or_array_refr(owned_named_types)).unwrap();
    writeln!(&mut out.aliases, ";").unwrap();
}

fn generate_seq_alias(out: &mut Output, name: &str, owned_named_type: &OwnedNamedType) {
    write!(&mut out.aliases, "pub type {} = Vec<", name).unwrap();
    write_ty_refr(&mut out.aliases, owned_named_type);
    writeln!(&mut out.aliases, ">;").unwrap();
}

fn generate_map_alias(out: &mut Output, name: &str, key: &OwnedNamedType, val: &OwnedNamedType) {
    // todo: always hashmap?
    write!(&mut out.aliases, "pub type {} = HashMap<", name).unwrap();
    write_ty_refr(&mut out.aliases, key);
    write!(&mut out.aliases, ", ").unwrap();
    write_ty_refr(&mut out.aliases, val);
    writeln!(&mut out.aliases, ">;").unwrap();
}

fn generate_schema_alias(out: &mut Output, name: &str) {
    writeln!(&mut out.aliases, "pub type {} = OwnedNamedType;", name).unwrap();
}

fn generate_enum(out: &mut Output, name: &str, owned_named_variants: &[OwnedNamedVariant]) {
    writeln!(&mut out.types, "#[derive(Serialize, Deserialize, Schema)]").unwrap();
    writeln!(&mut out.types, "pub enum {name} {{").unwrap();
    for v in owned_named_variants {
        write!(&mut out.types, "    {}", v.name).unwrap();
        match &v.ty {
            OwnedDataModelVariant::UnitVariant => {
                writeln!(&mut out.types, ",").unwrap();
            },
            OwnedDataModelVariant::NewtypeVariant(owned_named_type) => {
                write!(&mut out.types, "(").unwrap();
                write_ty_refr(&mut out.types, owned_named_type);
                writeln!(&mut out.types, "),").unwrap();
            },
            OwnedDataModelVariant::TupleVariant(owned_named_types) => {
                let mut items = vec![];
                for v in owned_named_types {
                    let mut s = String::new();
                    write_ty_refr(&mut s, v);
                    items.push(s);
                }
                let all = items.join(", ");
                writeln!(&mut out.types, "({all}),").unwrap();
            },
            OwnedDataModelVariant::StructVariant(owned_named_values) => {
                writeln!(&mut out.types, " {{").unwrap();
                for v in owned_named_values {
                    write!(&mut out.types, "        {}: ", v.name).unwrap();
                    write_ty_refr(&mut out.types, &v.ty);
                    writeln!(&mut out.types, ",").unwrap();
                }
                writeln!(&mut out.types, "    }},").unwrap();
            },
        }
    }
    writeln!(&mut out.types, "}}").unwrap();
    writeln!(&mut out.types).unwrap();
}

fn generate_unit_struct(out: &mut Output, name: &str) {
    writeln!(&mut out.types, "#[derive(Serialize, Deserialize, Schema)]").unwrap();
    writeln!(&mut out.types, "pub struct {name};").unwrap();
    writeln!(&mut out.types).unwrap();
}

fn generate_newtype_struct(out: &mut Output, name: &str, _ont: &OwnedNamedType) {
    writeln!(&mut out.types, "#[derive(Serialize, Deserialize, Schema)]").unwrap();
    // TODO: Load-bearing name! Name includes the newtype parts
    writeln!(&mut out.types, "pub struct {name};").unwrap();
    writeln!(&mut out.types).unwrap();
}

fn generate_tuple_struct(out: &mut Output, name: &str, _onvs: &[OwnedNamedType]) {
    writeln!(&mut out.types, "#[derive(Serialize, Deserialize, Schema)]").unwrap();
    // TODO: Load-bearing name! Name includes the newtype parts
    writeln!(&mut out.types, "pub struct {name};").unwrap();
    writeln!(&mut out.types).unwrap();
}

fn generate_struct(out: &mut Output, name: &str, fields: &[OwnedNamedValue]) {
    writeln!(&mut out.types, "#[derive(Serialize, Deserialize, Schema)]").unwrap();
    writeln!(&mut out.types, "pub struct {name} {{").unwrap();
    for f in fields {
        write!(&mut out.types, "    pub {}: ", f.name).unwrap();
        write_ty_refr(&mut out.types, &f.ty);
        writeln!(&mut out.types, ",").unwrap();
    }
    writeln!(&mut out.types, "}}").unwrap();
    writeln!(&mut out.types).unwrap();
}

fn write_ty_refr(out: &mut String, ont: &OwnedNamedType) {
    match &ont.ty {
        OwnedDataModelType::Bool => write!(out, "bool"),
        OwnedDataModelType::I8 => write!(out, "i8"),
        OwnedDataModelType::U8 => write!(out, "u8"),
        OwnedDataModelType::I16 => write!(out, "i16"),
        OwnedDataModelType::I32 => write!(out, "i32"),
        OwnedDataModelType::I64 => write!(out, "i64"),
        OwnedDataModelType::I128 => write!(out, "i128"),
        OwnedDataModelType::U16 => write!(out, "u16"),
        OwnedDataModelType::U32 => write!(out, "u32"),
        OwnedDataModelType::U64 => write!(out, "u64"),
        OwnedDataModelType::U128 => write!(out, "u128"),
        OwnedDataModelType::Usize => write!(out, "usize"),
        OwnedDataModelType::Isize => write!(out, "isize"),
        OwnedDataModelType::F32 => write!(out, "f32"),
        OwnedDataModelType::F64 => write!(out, "f64"),
        OwnedDataModelType::Char => write!(out, "char"),
        OwnedDataModelType::String => write!(out, "String"),
        OwnedDataModelType::ByteArray => write!(out, "Vec<u8>"),
        OwnedDataModelType::Unit => write!(out, "()"),
        OwnedDataModelType::Option(owned_named_type) => {
            write!(out, "Option<").unwrap();
            write_ty_refr(out, owned_named_type);
            write!(out, ">")
        },
        OwnedDataModelType::UnitStruct => write!(out, "{}", ont.name),
        OwnedDataModelType::NewtypeStruct(_owned_named_type) => {
            todo!("Known issue: since newtype names have the contained type in the name, we can't refer to them")
        },
        OwnedDataModelType::Seq(owned_named_type) => {
            write!(out, "Vec<").unwrap();
            write_ty_refr(out, owned_named_type);
            write!(out, ">")
        },
        OwnedDataModelType::Tuple(owned_named_types) => {
            write!(out, "{}", tuple_or_array_refr(owned_named_types))
        },
        OwnedDataModelType::TupleStruct(_owned_named_types) => {
            todo!("Known issue: since tuplestruct names have the contained type in the name, we can't refer to them")
        },
        OwnedDataModelType::Map { key, val } => {
            // todo: do we always want HashMap and not whatever other Map?
            write!(out, "HashMap<").unwrap();
            write_ty_refr(out, key);
            write!(out, ", ").unwrap();
            write_ty_refr(out, val);
            write!(out, ">")
        },
        OwnedDataModelType::Struct(_owned_named_values) => write!(out, "{}", ont.name),
        OwnedDataModelType::Enum(_owned_named_variants) => write!(out, "{}", ont.name),
        OwnedDataModelType::Schema => todo!(),
    }.unwrap();
}

fn tuple_or_array_refr(owned_named_types: &[OwnedNamedType]) -> String {
    let mut out = String::new();
    if let Some(font) = owned_named_types.first() {
        let multiple = owned_named_types.len() > 1;
        let all_same = owned_named_types.iter().all(|ont| ont.ty == font.ty);
        if multiple && all_same {
            // This is an array, not a tuple!
            write!(&mut out, "[").unwrap();
            write_ty_refr(&mut out, font);
            write!(&mut out, "; {}]", owned_named_types.len()).unwrap();
        } else {
            // This is really a tuple
            write!(&mut out, "(").unwrap();
            let mut items = vec![];
            for t in owned_named_types {
                let mut s = String::new();
                write_ty_refr(&mut s, t);
                items.push(s);
            }
            let all = items.join(", ");
            write!(&mut out, "{all})").unwrap();
        }
    } else {
        todo!("Tuple with no items?");
    }
    out
}
