//! Data structures for storing information about specifications.
//!
//! Please see the `parser.rs` file for more information about
//! specifications.

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{Display, Debug};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A specification type.
pub enum SpecType {
    /// Precondition of a procedure.
    Precondition,
    /// Postcondition of a procedure.
    Postcondition,
    /// Loop invariant or struct invariant
    Invariant,
    /// Predicate
    Predicate,
}

#[derive(Debug)]
/// A conversion from string into specification type error.
pub enum TryFromStringError {
    /// Reported when the string being converted is not one of the
    /// following: `requires`, `ensures`, `invariant`.
    UnknownSpecificationType,
}

impl<'a> TryFrom<&'a str> for SpecType {
    type Error = TryFromStringError;

    fn try_from(typ: &str) -> Result<SpecType, TryFromStringError> {
        match typ {
            "requires" => Ok(SpecType::Precondition),
            "ensures" => Ok(SpecType::Postcondition),
            "invariant" => Ok(SpecType::Invariant),
            "predicate" => Ok(SpecType::Predicate),
            _ => Err(TryFromStringError::UnknownSpecificationType),
        }
    }
}

#[derive(
    Debug, Default, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord,
)]
/// A unique ID of the specification element such as entire precondition
/// or postcondition.
pub struct SpecificationId(Uuid);

/// A reference to a procedure specification.
#[derive(Debug, Clone, Copy)]
pub enum SpecIdRef {
    Precondition(SpecificationId),
    Postcondition(SpecificationId),
    Pledge {
        lhs: Option<SpecificationId>,
        rhs: SpecificationId,
    },
    Predicate(SpecificationId),
}

impl Display for SpecificationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_simple().encode_lower(&mut Uuid::encode_buffer()),
        )
    }
}

impl std::convert::TryFrom<String> for SpecificationId {
    type Error = uuid::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Uuid::parse_str(&value).map(Self)
    }
}

impl SpecificationId {
    pub fn dummy() -> Self {
        Self(Uuid::nil())
    }
}

pub(crate) struct SpecificationIdGenerator {}

impl SpecificationIdGenerator {
    pub(crate) fn new() -> Self {
        Self {}
    }
    pub(crate) fn generate(&mut self) -> SpecificationId {
        SpecificationId(Uuid::new_v4())
    }
}

pub(crate) struct NameGenerator {}

impl NameGenerator {
    pub(crate) fn new() -> Self { Self { } }
    pub(crate) fn generate_struct_name(&self, item: &syn::ItemImpl) -> Result<String, String> {
        let name_ty = self.generate_name_for_type(&*item.self_ty)?;
        let uuid = Uuid::new_v4().to_simple();
        Ok(format!("PrustiStruct{}_{}", name_ty, uuid))
    }

    pub(crate) fn generate_struct_name_for_trait(&self, item: &syn::ItemTrait) -> String {
        let uuid = Uuid::new_v4().to_simple();
        format!("PrustiTrait{}_{}", item.ident, uuid)
    }

    pub(crate) fn generate_mod_name(&self, ident: &syn::Ident) -> String {
        let uuid = Uuid::new_v4().to_simple();
        format!("{}_{}", ident, uuid)
    }

    fn generate_name_for_type(&self, ty: &syn::Type) -> Result<String, String> {
        Ok(match ty {
            syn::Type::Path(ty_path) => {
                ty_path.path.segments.iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            },
            syn::Type::Slice(ty_slice) => {
                let ty = &*ty_slice.elem;
                format!("Slice{}", self.generate_name_for_type(ty)?.as_str())
            },
            _ => {
                return Err(format!("could not generate name for type {:?}", ty))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    mod name_generator {
        use crate::specifications::common::NameGenerator;
        use regex::Regex;

        const PATTERN: &str = r#"(.*)([0-9a-fA-F]{32})"#;

        #[test]
        fn generate_name_for_slice() {
            let item: syn::ItemImpl = syn::parse_quote!{impl [i32] {}};

            let name_generator = NameGenerator {};
            let name = name_generator.generate_struct_name(&item).unwrap();

            assert_uuid_prefix("PrustiStructSlicei32_", &name);
        }

        #[test]
        fn generate_name_for_path() {
            let item: syn::ItemImpl = syn::parse_quote!{impl std::option::Option<i32> {}};
            let name_generator = NameGenerator {};
            let name = name_generator.generate_struct_name(&item).unwrap();
            assert_uuid_prefix("PrustiStructstdoptionOption_", &name);
        }

        fn assert_uuid_prefix(prefix: &str, actual: &str) {
            let re = Regex::new(PATTERN).unwrap();
            let captures = re.captures(actual)
                .expect(format!("Regex '{}' did not match '{}'", PATTERN, actual).as_str());
            assert_eq!(3, captures.len());
            assert_eq!(prefix, captures.get(1).unwrap().as_str());
        }
    }
}