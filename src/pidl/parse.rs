//! PIDL Parser
//!
//! The goal of this file is to turn an input KDL file into a set of
//! `OwnedNameType`s, from `postcard-schema`, which we use as our
//! "intermediate representation".
//!
//! The general flow here is:
//!
//! KDL -> Unresolved Types -> OwnedNameTypes
//!
//! We first do one pass over the input, iterating using KDL nodes, gathering
//! "Unresolved" pieces, which are syntactic elements without having connected
//! them as final types. These can be `UnresolvedTypeDefn`, or items that are
//! *defining* a type, or `UnresolvedTypeRefr`, which are places where we are
//! *referencing* some other type.
//!
//! We then do "resolution", which attempts to convert all `UnresolvedTypeDefn`
//! into `OwnedNameTypes`, and convert all `UnresolvedTypeRefr` into the resolved
//! `OwnedNameTypes`.

use kdl::KdlNode;
use miette::SourceSpan;
use postcard_schema::{
    schema::owned::{
        OwnedDataModelType, OwnedDataModelVariant, OwnedNamedType, OwnedNamedValue,
        OwnedNamedVariant,
    },
    Schema,
};

use super::Error;

#[derive(Debug)]
pub struct PidlTypes {
    pub(crate) resolved: Vec<OwnedNamedType>,
}

impl PidlTypes {
    fn absorb_alias(node: &KdlNode) -> Result<UnresolvedTypeDefn<'_>, Error> {
        if let [name, ty] = node.entries() {
            let name = name
                .value()
                .as_string()
                .expect("alias should have two string args");
            let ty = ty
                .value()
                .as_string()
                .expect("alias should have two string args");

            Ok(UnresolvedTypeDefn::Alias {
                name,
                ty: UnresolvedTypeRefr::parse_entirely(ty)?,
                span: node.span(),
            })
        } else {
            todo!("alias should have two string args")
        }
    }

    fn absorb_struct_field(node: &KdlNode) -> Result<(&str, UnresolvedTypeRefr<'_>), Error> {
        let name = node.name().value();
        if let [ty] = node.entries() {
            let ty = ty
                .value()
                .as_string()
                .expect("struct field should have two string args");
            Ok((name, UnresolvedTypeRefr::parse_entirely(ty)?))
        } else {
            todo!()
        }
    }

    fn absorb_struct(node: &KdlNode) -> Result<UnresolvedTypeDefn<'_>, Error> {
        let entries = node.entries();
        let children = node.children();

        match (entries, children) {
            ([name], None) => {
                // UnitStruct
                let name = name.value().as_string().expect("struct needs a name");
                Ok(UnresolvedTypeDefn::UnitStruct {
                    name,
                    span: node.span(),
                })
            }
            ([name, ty], None) => {
                // newtypestruct/tuplestruct
                let name = name.value().as_string().expect("struct needs a name");
                let ty = ty.value().as_string().expect("ty needs a ty");
                Ok(UnresolvedTypeDefn::NewTypeTupleStruct {
                    name,
                    ty: UnresolvedTypeRefr::parse_entirely(ty)?,
                    span: node.span(),
                })
            }
            ([name], Some(children)) => {
                // struct
                let name = name.value().as_string().expect("struct needs a name");

                let mut fields = vec![];
                for ch in children.nodes() {
                    fields.push(Self::absorb_struct_field(ch)?);
                }

                // TODO: ensure all field names are unique and valid!

                Ok(UnresolvedTypeDefn::Struct {
                    name,
                    fields,
                    span: node.span(),
                })
            }
            _ => {
                panic!("What? {entries:?}, {children:?}");
            }
        }
    }

    fn absorb_enum(node: &KdlNode) -> Result<UnresolvedTypeDefn<'_>, Error> {
        let [name] = node.entries() else {
            panic!("What? {:?}", node.entries());
        };
        let name = name.value().as_string().expect("enum needs a name");
        let mut variants = vec![];
        for ch in node.iter_children() {
            variants.push(Self::absorb_enum_variant(ch)?);
        }

        // TODO: ensure all variant names are unique and valid!

        Ok(UnresolvedTypeDefn::Enum {
            name,
            variants,
            span: node.span(),
        })
    }

    fn absorb_enum_variant(node: &KdlNode) -> Result<UnresolvedEnumVariant<'_>, Error> {
        let name = node.name().value();
        let entries = node.entries();
        let children = node.children();

        match (entries, children) {
            ([], None) => Ok(UnresolvedEnumVariant::Unit { name }),
            ([], Some(children)) => {
                let mut fields = vec![];
                for ch in children.nodes() {
                    fields.push(Self::absorb_struct_field(ch)?);
                }

                Ok(UnresolvedEnumVariant::Struct { name, fields })
            }
            ([ty], None) => {
                let ty = ty.value().as_string().expect("ty needs a ty");
                let item = UnresolvedTypeRefr::parse_entirely(ty)?;
                if let UnresolvedTypeRefr::Tuple { tys } = item {
                    Ok(UnresolvedEnumVariant::Tuple { name, fields: tys })
                } else {
                    Ok(UnresolvedEnumVariant::NewType { name, ty: item })
                }
            }
            _ => todo!("What? entries: {entries:?}, children: {children:?}"),
        }
    }

    pub fn from_node(node: &KdlNode) -> Result<Self, Error> {
        assert!(node.entries().is_empty());

        let mut types = vec![];

        for ch in node.iter_children() {
            match ch.name().value() {
                "alias" => {
                    types.push(Self::absorb_alias(ch)?);
                }
                "struct" => {
                    types.push(Self::absorb_struct(ch)?);
                }
                "enum" => {
                    types.push(Self::absorb_enum(ch)?);
                }
                other => todo!("what: '{other}'"),
            }
        }

        // TODO: Actually resolve types!
        let mut rtypes = vec![];
        resolve_types(&mut rtypes, &mut types)?;

        Ok(Self { resolved: rtypes })
    }
}

#[derive(Debug)]
enum UnresolvedTypeRefr<'a> {
    Name {
        name: &'a str,
    },
    Option {
        ty: Box<UnresolvedTypeRefr<'a>>,
    },
    Seq {
        ty: Box<UnresolvedTypeRefr<'a>>,
    },
    Array {
        ty: Box<UnresolvedTypeRefr<'a>>,
        ct: usize,
    },
    Map {
        kty: Box<UnresolvedTypeRefr<'a>>,
        vty: Box<UnresolvedTypeRefr<'a>>,
    },
    Tuple {
        tys: Vec<UnresolvedTypeRefr<'a>>,
    },
}

impl<'a> UnresolvedTypeRefr<'a> {
    fn parse_entirely(s: &'a str) -> Result<Self, Error> {
        let (me, rem) = Self::parse(s)?;
        if rem.trim().is_empty() {
            Ok(me)
        } else {
            Err(Error::Invalid(rem.to_string()))
        }
    }

    fn parse(s: &'a str) -> Result<(Self, &'a str), Error> {
        if s.starts_with('(') {
            Self::parse_tuple(s)
        } else if s.starts_with('[') {
            Self::parse_seq_array(s)
        } else if s.starts_with("option<") {
            Self::parse_option(s)
        } else if s.starts_with("map<") {
            Self::parse_map(s)
        } else if let Ok((tyn, rem)) = parser::take_valid_rust_tyname(s) {
            Ok((Self::Name { name: tyn }, rem))
        } else {
            todo!("what: {s}")
        }
    }

    fn parse_tuple(s: &'a str) -> Result<(UnresolvedTypeRefr<'a>, &'a str), Error> {
        let mut remain = parser::take_char(s, '(').map_err(|s| Error::Invalid(s.to_string()))?;
        let mut items = vec![];
        loop {
            if let Ok(rem) = parser::take_char(remain, ')') {
                remain = rem;
                break;
            } else if let Ok(rem) = parser::take_char(remain, ',') {
                remain = rem;
            } else if let Ok((ty, rem)) = Self::parse(remain) {
                remain = rem;
                items.push(ty);
            } else {
                return Err(Error::Invalid(remain.to_string()));
            }
        }
        Ok((UnresolvedTypeRefr::Tuple { tys: items }, remain))
    }

    fn parse_seq_array(s: &'a str) -> Result<(UnresolvedTypeRefr<'a>, &'a str), Error> {
        let remain = parser::take_char(s, '[').map_err(|s| Error::Invalid(s.to_string()))?;
        let (ty, remain) = Self::parse(remain)?;
        if let Ok(remain) = parser::take_char(remain, ']') {
            return Ok((UnresolvedTypeRefr::Seq { ty: Box::new(ty) }, remain));
        }
        let Ok(remain) = parser::take_char(remain, ';') else {
            return Err(Error::Invalid(remain.to_string()));
        };
        let Ok((ct, remain)) = parser::take_num(remain) else {
            return Err(Error::Invalid(remain.to_string()));
        };
        if let Ok(remain) = parser::take_char(remain, ']') {
            Ok((
                UnresolvedTypeRefr::Array {
                    ty: Box::new(ty),
                    ct,
                },
                remain,
            ))
        } else {
            Err(Error::Invalid(remain.to_string()))
        }
    }

    fn parse_option(s: &'a str) -> Result<(UnresolvedTypeRefr<'a>, &'a str), Error> {
        let remain = parser::take_str(s, "option").map_err(|s| Error::Invalid(s.to_string()))?;
        let remain = parser::take_char(remain, '<').map_err(|s| Error::Invalid(s.to_string()))?;
        let (ty, remain) = Self::parse(remain)?;
        let remain = parser::take_char(remain, '>').map_err(|s| Error::Invalid(s.to_string()))?;
        Ok((UnresolvedTypeRefr::Option { ty: Box::new(ty) }, remain))
    }

    fn parse_map(s: &'a str) -> Result<(UnresolvedTypeRefr<'a>, &'a str), Error> {
        let remain = parser::take_str(s, "map").map_err(|s| Error::Invalid(s.to_string()))?;
        let remain = parser::take_char(remain, '<').map_err(|s| Error::Invalid(s.to_string()))?;
        let (kty, remain) = Self::parse(remain)?;
        let remain = parser::take_char(remain, ',').map_err(|s| Error::Invalid(s.to_string()))?;
        let (vty, remain) = Self::parse(remain)?;
        let remain = parser::take_char(remain, '>').map_err(|s| Error::Invalid(s.to_string()))?;
        Ok((
            UnresolvedTypeRefr::Map {
                kty: Box::new(kty),
                vty: Box::new(vty),
            },
            remain,
        ))
    }
}

/// Simple helper methods, mostly for taking/skipping certain syntax elements
mod parser {
    use super::is_valid_rust_tyname;

    pub fn take_char(s: &str, c: char) -> Result<&str, &str> {
        let trim = s.trim_start();
        if let Some(rem) = trim.strip_prefix(c) {
            Ok(rem)
        } else {
            Err(trim)
        }
    }

    pub fn take_str<'a>(s: &'a str, t: &str) -> Result<&'a str, &'a str> {
        let trim = s.trim_start();
        if let Some(rem) = trim.strip_prefix(t) {
            Ok(rem)
        } else {
            Err(trim)
        }
    }

    pub(crate) fn take_num(s: &str) -> Result<(usize, &str), &str> {
        // todo: much smarter
        let trim = s.trim_start();
        let (trim, remain) = {
            let nums = trim.split_once(|c: char| !(c.is_numeric() || c == '_'));
            if let Some((st, _)) = nums {
                let (now, later) = trim.split_at(st.len());
                (now, later)
            } else {
                (trim, "")
            }
        };
        trim.parse::<usize>().map_err(|_| trim).map(|n| (n, remain))
    }

    pub(crate) fn take_valid_rust_tyname(s: &str) -> Result<(&str, &str), &str> {
        // wrong
        let trim = s.trim();
        let tyn = trim.split_once(|c: char| !c.is_ascii_alphanumeric() && c != '_');
        if let Some((st, _)) = tyn {
            let (now, later) = trim.split_at(st.len());
            Ok((now, later))
        } else if is_valid_rust_tyname(trim) {
            Ok((trim, ""))
        } else {
            Err(trim)
        }
    }
}

fn is_valid_rust_tyname(s: &str) -> bool {
    // this is wrong
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(Debug)]
enum UnresolvedEnumVariant<'a> {
    Unit {
        name: &'a str,
    },
    NewType {
        name: &'a str,
        ty: UnresolvedTypeRefr<'a>,
    },
    Tuple {
        name: &'a str,
        fields: Vec<UnresolvedTypeRefr<'a>>,
    },
    Struct {
        name: &'a str,
        fields: Vec<(&'a str, UnresolvedTypeRefr<'a>)>,
    },
}

#[derive(Debug)]
enum UnresolvedTypeDefn<'a> {
    Alias {
        name: &'a str,
        ty: UnresolvedTypeRefr<'a>,
        span: SourceSpan,
    },
    UnitStruct {
        name: &'a str,
        span: SourceSpan,
    },
    NewTypeTupleStruct {
        name: &'a str,
        ty: UnresolvedTypeRefr<'a>,
        span: SourceSpan,
    },
    Struct {
        name: &'a str,
        fields: Vec<(&'a str, UnresolvedTypeRefr<'a>)>,
        span: SourceSpan,
    },
    Enum {
        name: &'a str,
        variants: Vec<UnresolvedEnumVariant<'a>>,
        span: SourceSpan,
    },
}

const BUILTIN_TYPE_NAMES: &[&str] = &[
    "bool",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "f32",
    "f64",
    "char",
    "string",
    "bytearray",
    "unit",
    "option",
    "unitstruct",
    "newtypestruct",
    "seq",
    "tuple",
    "tuplestruct",
    "map",
    "struct",
    "enum",
];

fn resolve_ty(
    ty: &UnresolvedTypeRefr<'_>,
    known: &[OwnedNamedType],
) -> Result<Option<OwnedNamedType>, Error> {
    // println!("Resolving '{name}'");
    match ty {
        UnresolvedTypeRefr::Name { name } => match *name {
            "bool" => Ok(Some(<bool as Schema>::SCHEMA.into())),
            "i8" => Ok(Some(<i8 as Schema>::SCHEMA.into())),
            "i16" => Ok(Some(<i16 as Schema>::SCHEMA.into())),
            "i32" => Ok(Some(<i32 as Schema>::SCHEMA.into())),
            "i64" => Ok(Some(<i64 as Schema>::SCHEMA.into())),
            "i128" => Ok(Some(<i128 as Schema>::SCHEMA.into())),
            "u8" => Ok(Some(<u8 as Schema>::SCHEMA.into())),
            "u16" => Ok(Some(<u16 as Schema>::SCHEMA.into())),
            "u32" => Ok(Some(<u32 as Schema>::SCHEMA.into())),
            "u64" => Ok(Some(<u64 as Schema>::SCHEMA.into())),
            "u128" => Ok(Some(<u128 as Schema>::SCHEMA.into())),
            "f32" => Ok(Some(<f32 as Schema>::SCHEMA.into())),
            "f64" => Ok(Some(<f64 as Schema>::SCHEMA.into())),
            "char" => Ok(Some(<char as Schema>::SCHEMA.into())),
            "string" => Ok(Some(<&str as Schema>::SCHEMA.into())),
            "bytearray" => Ok(Some(<[u8] as Schema>::SCHEMA.into())),
            "unit" | "()" => Ok(Some(<() as Schema>::SCHEMA.into())),
            "option" => Err(Error::Invalid("don't do that".into())),
            "unitstruct" => Err(Error::Invalid("don't do that".into())),
            "newtypestruct" => Err(Error::Invalid("don't do that".into())),
            "seq" => Err(Error::Invalid("don't do that".into())),
            "tuple" => Err(Error::Invalid("don't do that".into())),
            "tuplestruct" => Err(Error::Invalid("don't do that".into())),
            "map" => Err(Error::Invalid("don't do that".into())),
            "struct" => Err(Error::Invalid("don't do that".into())),
            "enum" => Err(Error::Invalid("don't do that".into())),
            _ => {
                if looks_adhoc(name) {
                    todo!("adhoc: '{name}'")
                } else {
                    Ok(known.iter().find(|ont| ont.name.as_str() == *name).cloned())
                }
            }
        },
        UnresolvedTypeRefr::Option { ty } => match resolve_ty(ty, known)? {
            Some(t) => Ok(Some(OwnedNamedType {
                name: format!("Option<{}>", t.name),
                ty: OwnedDataModelType::Option(Box::new(t)),
            })),
            None => Ok(None),
        },
        UnresolvedTypeRefr::Seq { ty } => match resolve_ty(ty, known)? {
            Some(t) => Ok(Some(OwnedNamedType {
                name: format!("[{}]", t.name),
                ty: OwnedDataModelType::Seq(Box::new(t)),
            })),
            None => Ok(None),
        },
        UnresolvedTypeRefr::Array { ty, ct } => match resolve_ty(ty, known)? {
            Some(t) => Ok(Some(OwnedNamedType {
                name: format!("[{}; {ct}]", t.name),
                ty: OwnedDataModelType::Tuple({
                    let mut v = vec![];
                    for _ in 0..*ct {
                        v.push(t.clone());
                    }
                    v
                }),
            })),
            None => Ok(None),
        },
        UnresolvedTypeRefr::Map { kty, vty } => {
            let a = resolve_ty(kty, known)?;
            let b = resolve_ty(vty, known)?;
            let (Some(k), Some(v)) = (a, b) else {
                return Ok(None);
            };
            Ok(Some(OwnedNamedType {
                name: format!("Map<{}, {}>", k.name, v.name),
                ty: OwnedDataModelType::Map {
                    key: Box::new(k),
                    val: Box::new(v),
                },
            }))
        }
        UnresolvedTypeRefr::Tuple { tys } => {
            let mut ts = vec![];
            let mut names = vec![];
            for t in tys {
                let t = resolve_ty(t, known)?;
                let Some(t) = t else {
                    return Ok(None);
                };
                names.push(t.name.clone());
                ts.push(t);
            }
            let joined = names.join(", ");
            Ok(Some(OwnedNamedType {
                name: format!("({joined})"),
                ty: OwnedDataModelType::Tuple(ts),
            }))
        }
    }
}

fn looks_adhoc(name: &str) -> bool {
    name.starts_with('(')
        || name.starts_with("option<")
        || name.starts_with('[')
        || name.starts_with("map<")
}

fn new_tyname_legal(name: &str, known: &[OwnedNamedType]) -> bool {
    // does this name alias a builtin?
    if BUILTIN_TYPE_NAMES.contains(&name) {
        return false;
    }
    // does this name alias an existing type?
    if known.iter().any(|ont| ont.name.as_str() == name) {
        return false;
    }
    // TODO: valid Rust type name check
    true
}

impl UnresolvedTypeDefn<'_> {
    fn resolve(&self, known: &[OwnedNamedType]) -> Result<Option<OwnedNamedType>, Error> {
        match self {
            UnresolvedTypeDefn::Alias { name, ty, span } => {
                Self::resolve_alias(name, ty, span, known)
            }
            UnresolvedTypeDefn::Struct { name, fields, span } => {
                Self::resolve_struct(name, fields, span, known)
            }
            UnresolvedTypeDefn::UnitStruct { name, span } => {
                Self::resolve_unitstruct(name, span, known)
            }
            UnresolvedTypeDefn::NewTypeTupleStruct { name, ty, span } => {
                Self::resolve_newtype_tuple_struct(name, ty, span, known)
            }
            UnresolvedTypeDefn::Enum {
                name,
                variants,
                span,
            } => Self::resolve_enum(name, variants, span, known),
        }
    }

    fn resolve_alias(
        name: &str,
        ty: &UnresolvedTypeRefr<'_>,
        _span: &SourceSpan,
        known: &[OwnedNamedType],
    ) -> Result<Option<OwnedNamedType>, Error> {
        // Is the name illegal?
        if !new_tyname_legal(name, known) {
            return Err(Error::BadName(name.to_string()));
        }
        match resolve_ty(ty, known) {
            Ok(Some(t)) => Ok(Some(OwnedNamedType {
                name: name.to_string(),
                ty: t.ty,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn resolve_unitstruct(
        name: &str,
        _span: &SourceSpan,
        known: &[OwnedNamedType],
    ) -> Result<Option<OwnedNamedType>, Error> {
        if !new_tyname_legal(name, known) {
            return Err(Error::BadName(name.to_string()));
        }
        Ok(Some(OwnedNamedType {
            name: name.to_string(),
            ty: OwnedDataModelType::UnitStruct,
        }))
    }

    fn resolve_newtype_tuple_struct(
        name: &str,
        ty: &UnresolvedTypeRefr<'_>,
        _span: &SourceSpan,
        known: &[OwnedNamedType],
    ) -> Result<Option<OwnedNamedType>, Error> {
        if !new_tyname_legal(name, known) {
            return Err(Error::BadName(name.to_string()));
        }
        let t = match resolve_ty(ty, known) {
            Ok(Some(t)) => t,
            Ok(None) => return Ok(None),
            Err(_) => todo!(),
        };

        if let OwnedNamedType {
            name: tname,
            ty: OwnedDataModelType::Tuple(t),
        } = t
        {
            Ok(Some(OwnedNamedType {
                name: format!("{name}{tname}"),
                ty: OwnedDataModelType::TupleStruct(t),
            }))
        } else {
            Ok(Some(OwnedNamedType {
                name: format!("{name}({})", t.name),
                ty: OwnedDataModelType::NewtypeStruct(Box::new(t)),
            }))
        }
    }

    fn resolve_struct(
        name: &str,
        fields: &[(&str, UnresolvedTypeRefr<'_>)],
        _span: &SourceSpan,
        known: &[OwnedNamedType],
    ) -> Result<Option<OwnedNamedType>, Error> {
        if !new_tyname_legal(name, known) {
            return Err(Error::BadName(name.to_string()));
        }
        let mut rfields = vec![];
        for (fname, fty) in fields {
            // todo: check field name legal
            match resolve_ty(fty, known) {
                Ok(Some(t)) => {
                    rfields.push(OwnedNamedValue {
                        name: fname.to_string(),
                        ty: t,
                    });
                }
                Ok(None) => return Ok(None),
                Err(_) => todo!(),
            }
        }

        // TODO: make sure there are no dupes in field names

        if rfields.is_empty() {
            todo!("We should have caught this earlier")
            // Ok(Some(OwnedNamedType {
            //     name: name.to_string(),
            //     ty: OwnedDataModelType::UnitStruct,
            // }))
        } else {
            Ok(Some(OwnedNamedType {
                name: name.to_string(),
                ty: OwnedDataModelType::Struct(rfields),
            }))
        }
    }

    fn resolve_enum(
        name: &str,
        variants: &[UnresolvedEnumVariant<'_>],
        _span: &SourceSpan,
        known: &[OwnedNamedType],
    ) -> Result<Option<OwnedNamedType>, Error> {
        if !new_tyname_legal(name, known) {
            return Err(Error::BadName(name.to_string()));
        }
        let mut rvars = vec![];
        for var in variants {
            match var {
                UnresolvedEnumVariant::Unit { name } => {
                    rvars.push(OwnedNamedVariant {
                        name: name.to_string(),
                        ty: OwnedDataModelVariant::UnitVariant,
                    });
                }
                UnresolvedEnumVariant::NewType { name, ty } => {
                    let Some(t) = resolve_ty(ty, known)? else {
                        return Ok(None);
                    };
                    rvars.push(OwnedNamedVariant {
                        name: name.to_string(),
                        ty: OwnedDataModelVariant::NewtypeVariant(Box::new(t)),
                    });
                }
                UnresolvedEnumVariant::Tuple { name, fields } => {
                    let mut rfields = vec![];
                    for f in fields {
                        let Some(t) = resolve_ty(f, known)? else {
                            return Ok(None);
                        };
                        rfields.push(t);
                    }
                    rvars.push(OwnedNamedVariant {
                        name: name.to_string(),
                        ty: OwnedDataModelVariant::TupleVariant(rfields),
                    });
                }
                UnresolvedEnumVariant::Struct { name, fields } => {
                    let mut rfields = vec![];
                    for (n, ty) in fields {
                        let Some(t) = resolve_ty(ty, known)? else {
                            return Ok(None);
                        };
                        rfields.push(OwnedNamedValue {
                            name: n.to_string(),
                            ty: t,
                        });
                    }
                    rvars.push(OwnedNamedVariant {
                        name: name.to_string(),
                        ty: OwnedDataModelVariant::StructVariant(rfields),
                    });
                }
            }
        }
        Ok(Some(OwnedNamedType {
            name: name.to_string(),
            ty: OwnedDataModelType::Enum(rvars),
        }))
    }
}

fn resolve_types(
    known: &mut Vec<OwnedNamedType>,
    unknown: &mut Vec<UnresolvedTypeDefn<'_>>,
) -> Result<(), Error> {
    // This is potentially the worst way to do this. We loop over and over as
    // long as we keep resolving SOMETHING.
    while !unknown.is_empty() {
        let mut progress = false;
        let mut old_unks = core::mem::take(unknown);
        for unk in old_unks.drain(..) {
            if let Some(k) = unk.resolve(known)? {
                known.push(k);
                progress = true;
            } else {
                unknown.push(unk);
            }
        }
        if !progress {
            todo!()
        }
    }
    Ok(())
}
