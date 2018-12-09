use syn::{
    Ident,
    Type,
    TypePath,
    Path,
    PathSegment,
    PathArguments,
    AngleBracketedGenericArguments,
    GenericArgument,
    Field,
};

/// Returns the inner type of the Option if the given path represents the Option type
fn inner_option_type(path: &Path) -> Option<&Type> {
    match path {
        // This is a naive test
        Path {leading_colon: None, segments} if segments.len() == 1 => {
            // Safe unwrap because we already checked the length
            let last = segments.last().unwrap().into_value();
            match last {
                PathSegment {
                    ident: type_name,
                    arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: None,
                        args,
                        ..
                    }),
                } if type_name == "Option" && args.len() == 1 => {
                    match args.last().unwrap().into_value() {
                        GenericArgument::Type(ty) => Some(ty),
                        _ => None,
                    }
                },
                _ => None,
            }
        },
        _ => None,
    }
}

/// One of the Components in a group, potentially optional
///
/// The ty field of this struct is assumed to implement Component
/// is_optional represents that this type may not be present in the World and that we should
/// store None if that is the case
#[derive(Debug)]
pub struct ComponentField<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub is_optional: bool,
}

impl<'a> From<&'a Field> for ComponentField<'a> {
    fn from(Field {ident, ty, ..}: &'a Field) -> Self {
        let (ty, is_optional) = match ty {
            // Matching Option is not very sophisticated here. We just look for a type == "Option"
            // That means that using the fully-qualified name would fail.
            Type::Path(TypePath {
                qself: None,
                path,
            }) => match inner_option_type(path) {
                Some(ty) => (ty, true),
                _ => (ty, false),
            },
            _ => (ty, false),
        };

        Self {
            // Fields from NamedFields always have field names
            ident: ident.as_ref().unwrap(),
            ty,
            is_optional,
        }
    }
}
