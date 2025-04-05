use parse::PidlTypes;
use postcard_schema::schema::owned::OwnedNamedType;
use thiserror::Error;

mod parse;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse Error")]
    Parse(#[from] kdl::KdlError),

    #[error("Validation: {0}")]
    Invalid(String),

    #[error("No Types were found")]
    NoTypes,

    #[error("Illegal type name: {0}")]
    BadName(String),
}

/// A postcard-idl record
///
/// Parsed from IDL files
pub struct Pidl {
    pub types: Vec<OwnedNamedType>,
}

impl Pidl {
    /// Attempt to parse a [`Pidl`] from the given `str`.
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

        let Some(types) = types else {
            return Err(Error::NoTypes);
        };

        Ok(Self {
            types: types.resolved,
        })
    }
}
