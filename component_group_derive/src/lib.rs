//! The custom derive for the `ComponentGroup` trait is defined in this crate. There is no need to
//! depend on this crate because its contents are automatically re-exported by the
//! `component_group` crate. See the documentation of that crate for more information.

extern crate proc_macro;

mod component_field;

use syn::{
    DeriveInput,
    Data,
    DataStruct,
    DataEnum,
    DataUnion,
    Fields,
    Ident,
    Generics,
    FieldsNamed,
    Field,
    parse_macro_input,
    token::{Struct, Enum, Union},
};
use proc_macro2::{TokenStream, Span};
use quote::quote;

use crate::component_field::ComponentField;

#[proc_macro_derive(ComponentGroup)]
pub fn derive_component_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let DeriveInput {ident, generics, data, ..} = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(DataStruct {
            struct_token: Struct {span},
            fields: Fields::Named(FieldsNamed {named: fields, ..}),
            ..
        }) => {
            if fields.is_empty() {
                error(span, "struct must have at least one field to derive ComponentGroup")
            } else {
                impl_component_group(ident, &generics, fields.iter())
            }.into()
        },
        Data::Struct(DataStruct {struct_token: Struct {span}, ..}) |
        Data::Enum(DataEnum {enum_token: Enum {span}, ..}) |
        Data::Union(DataUnion {union_token: Union {span}, ..}) => {
            error(span, "Only structs with named fields are supported").into()
        },
    }
}

fn error(span: Span, message: &str) -> TokenStream {
    syn::Error::new(span, message).to_compile_error()
}

/// Generates an impl of the ComponentGroup trait for the given struct
fn impl_component_group<'a>(
    ident: Ident,
    generics: &'a Generics,
    fields: impl Iterator<Item=&'a Field>,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields: Vec<_> = fields.map(ComponentField::from).collect();
    let field_names: Vec<_> = fields.iter().map(|f| f.ident).collect();
    let first_from_world = first_from_world_method(&field_names, &fields);
    let from_world = from_world_method(&field_names, &fields);
    let create = create_method(&fields);
    let update = update_method(&field_names, &fields);
    let remove = remove_method(&field_names, &fields);
    quote! {
        impl #impl_generics component_group::ComponentGroup for #ident #ty_generics #where_clause {
            #first_from_world
            #from_world
            #create
            #update
            #remove
        }
    }
}

fn first_from_world_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let joinables = fields.into_iter().map(|&ComponentField {ident: field_name, is_optional, ..}| {
        if is_optional {
            quote! {#field_name.maybe()}
        } else {
            quote! {&#field_name}
        }
    });
    let clones = fields.into_iter().map(|&ComponentField {ident: field_name, is_optional, ..}| {
        if is_optional {
            quote! {#field_name.cloned()}
        } else {
            quote! {Clone::clone(#field_name)}
        }
    });
    let tys = fields.into_iter().map(|f| f.ty);
    quote! {
        fn first_from_world(world: &specs::World) -> Option<(specs::Entity, Self)> {
            use specs::{Join, Entities};
            let ( __entities, #(#field_names),* ) = world.system_data::<( Entities, #(specs::ReadStorage<#tys>),* )>();
            ( &__entities, #(#joinables),* ).join().next().map(|( __entity, #(#field_names),* )| (__entity, Self {
                #(#field_names : #clones),*
            }))
        }
    }
}

fn from_world_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let tys = fields.into_iter().map(|f| f.ty);
    let reads = fields.into_iter().map(|&ComponentField {ident: field_name, ty, is_optional}| {
        if is_optional {
            quote! {#field_name.get(entity).cloned()}
        } else {
            let err = format!("expected a {} component to be present", quote!(#ty));
            quote! {#field_name.get(entity).cloned().expect(#err)}
        }
    });
    quote! {
        fn from_world(entity: specs::Entity, world: &specs::World) -> Self {
            let ( #(#field_names),* ) = world.system_data::<( #(specs::ReadStorage<#tys>),* )>();

            Self {
                #( #field_names : #reads ),*
            }
        }
    }
}

fn create_method(fields: &[ComponentField]) -> TokenStream {
    let with_comp = fields.into_iter().map(|&ComponentField {ident: field_name, is_optional, ..}| {
        if is_optional {
            quote! {
                if let Some(#field_name) = self.#field_name {
                    builder = builder.with(#field_name);
                }
            }
        } else {
            quote! { builder = builder.with(self.#field_name); }
        }
    });
    quote! {
        fn create(self, world: &mut specs::World) -> specs::Entity {
            use specs::Builder;
            #[allow(unused_mut)]
            let mut builder = world.create_entity();
            #( #with_comp )*
            builder.build()
        }
    }
}

fn update_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let tys = fields.into_iter().map(|f| f.ty);
    let updates = fields.into_iter().map(|&ComponentField {ident: field_name, is_optional, ..}| {
        if is_optional {
            quote! {
                match self.#field_name {
                    Some(value) => #field_name.insert(entity, value)?,
                    None => #field_name.remove(entity),
                };
            }
        } else {
            quote! { #field_name.insert(entity, self.#field_name)?; }
        }
    });
    quote! {
        type UpdateError = specs::error::Error;
        fn update(self, entity: specs::Entity, world: &mut specs::World) -> Result<(), Self::UpdateError> {
            let ( #(mut #field_names),* ) = world.system_data::<( #( specs::WriteStorage<#tys> ),* )>();

            #( #updates )*

            Ok(())
        }
    }
}

fn remove_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let tys = fields.into_iter().map(|f| f.ty);
    let reads = fields.into_iter().map(|&ComponentField {ident: field_name, ty, is_optional}| {
        if is_optional {
            quote! {#field_name.remove(entity)}
        } else {
            let err = format!("expected a {} component to be present", quote!(#ty));
            quote! {#field_name.remove(entity).expect(#err)}
        }
    });
    quote! {
        fn remove(entity: specs::Entity, world: &mut specs::World) -> Self {
            let ( #(mut #field_names),* ) = world.system_data::<( #(specs::WriteStorage<#tys>),* )>();

            Self {
                #( #field_names : #reads ),*
            }
        }
    }
}
