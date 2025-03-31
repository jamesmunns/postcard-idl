use kdl::KdlNode;
use miette::SourceSpan;
use postcard_schema::{schema::owned::OwnedNamedType, Schema};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse Error")]
    Parse(#[from] kdl::KdlError),

    #[error("Validation: {0}")]
    Invalid(String),
}

pub struct Pidl {}

impl Pidl {
    pub fn parse_from_str(s: &str) -> Result<Self, Error> {
        let doc = kdl::KdlDocument::parse(s)?;
        let mut types = None;

        for x in doc.nodes() {
            match x.name().value() {
                "types" => {
                    let t = PidlTypes::from_node(x)?;
                    assert!(types.replace(t).is_none());
                }
                _ => todo!(),
            }
        }

        println!("Types: {types:#?}");

        Ok(Self {})
    }
}

#[derive(Debug)]
enum UnresolvedType<'a> {
    Alias {
        name: &'a str,
        ty: &'a str,
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

fn resolve_ty_by_name(
    name: &str,
    known: &[OwnedNamedType],
) -> Result<Option<OwnedNamedType>, Error> {
    match name {
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
        _ => Ok(known.iter().find(|ont| ont.name.as_str() == name).cloned()),
    }
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

impl<'a> UnresolvedType<'a> {
    fn resolve(&self, known: &[OwnedNamedType]) -> Result<Option<OwnedNamedType>, Error> {
        match self {
            UnresolvedType::Alias { name, ty, span } => Self::resolve_alias(name, ty, span, known),
        }
    }

    fn resolve_alias(
        name: &str,
        ty: &str,
        span: &SourceSpan,
        known: &[OwnedNamedType],
    ) -> Result<Option<OwnedNamedType>, Error> {
        // Is the name illegal?
        if !new_tyname_legal(name, known) {
            return Err(todo!());
        }
        match resolve_ty_by_name(ty, known) {
            Ok(Some(t)) => Ok(Some(OwnedNamedType {
                name: name.to_string(),
                ty: t.ty,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

fn resolve_types(
    known: &mut Vec<OwnedNamedType>,
    unknown: &mut Vec<UnresolvedType<'_>>,
) -> Result<(), Error> {
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

#[derive(Debug)]
pub struct PidlTypes {
    resolved: Vec<OwnedNamedType>,
}

impl PidlTypes {
    pub fn from_node(node: &KdlNode) -> Result<Self, Error> {
        assert!(node.entries().is_empty());

        let mut types = vec![];

        for ch in node.iter_children() {
            match ch.name().value() {
                "alias" => {
                    if let [name, ty] = ch.entries() {
                        let name = name
                            .value()
                            .as_string()
                            .expect("alias should have two string args");
                        let ty = ty
                            .value()
                            .as_string()
                            .expect("alias should have two string args");

                        types.push(UnresolvedType::Alias {
                            name,
                            ty,
                            span: ch.span(),
                        });
                    } else {
                        todo!("alias should have two string args")
                    }
                }
                _ => todo!("what"),
            }
        }

        // TODO: Actually resolve types!
        let mut rtypes = vec![];
        resolve_types(&mut rtypes, &mut types)?;


        Ok(Self { resolved: rtypes })
    }
}
