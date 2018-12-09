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
use proc_macro2::TokenStream;
use quote::quote;

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

impl ComponentField<'_> {
    /// Returns the code to make this into a type that can be used with the Join trait
    pub fn joinable(&self) -> TokenStream {
        let field_name = self.ident;
        if self.is_optional {
            quote! {#field_name.maybe()}
        } else {
            quote! {&#field_name}
        }
    }

    /// Returns the code to clone a fetched value of this field
    pub fn cloned(&self) -> TokenStream {
        let field_name = self.ident;
        if self.is_optional {
            quote! {#field_name.cloned()}
        } else {
            quote! {Clone::clone(#field_name)}
        }
    }

    /// Returns the code to read this field from the storage
    ///
    /// If the field is not optional, this will also add a call to expect() that ensures that the
    /// field was actually there
    pub fn read_value(&self) -> TokenStream {
        let field_name = self.ident;
        if self.is_optional {
            quote! {#field_name.get(entity).cloned()}
        } else {
            let ty = self.ty;
            let err = format!("bug: expected a {} component to be present", quote!(#ty));
            quote! {#field_name.get(entity).cloned().expect(#err)}
        }
    }

    pub fn add_to_builder(&self) -> TokenStream {
        let field_name = self.ident;
        if self.is_optional {
            quote! {
                if let Some(#field_name) = self.#field_name {
                    builder = builder.with(#field_name);
                }
            }
        } else {
            quote! { builder = builder.with(self.#field_name); }
        }
    }

    pub fn update_value(&self) -> TokenStream {
        let field_name = self.ident;
        if self.is_optional {
            quote! {
                match self.#field_name {
                    Some(value) => #field_name.insert(entity, value)?,
                    None => #field_name.remove(entity),
                };
            }
        } else {
            quote! { #field_name.insert(entity, self.#field_name)?; }
        }
    }
}
