extern crate proc_macro;

mod component_field;

use syn::{
    parse_macro_input,
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
    token::{Struct, Enum, Union},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::component_field::ComponentField;

#[proc_macro_derive(ComponentGroup)]
pub fn derive_component_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let DeriveInput {ident, generics, data, ..} = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(DataStruct {fields: Fields::Named(FieldsNamed {named: fields, ..}), ..}) => {
            impl_component_group(ident, &generics, fields.iter()).into()
        },
        Data::Struct(DataStruct {struct_token: Struct {span}, ..}) |
        Data::Enum(DataEnum {enum_token: Enum {span}, ..}) |
        Data::Union(DataUnion {union_token: Union {span}, ..}) => {
            syn::Error::new(span, "Only structs with named fields are supported")
                .to_compile_error()
                .into()
        },
    }
}

/// Generates an impl of the ComponentGroup trait for the given struct
fn impl_component_group<'a>(
    ident: Ident,
    generics: &'a Generics,
    fields: impl Iterator<Item=&'a Field>,
) -> TokenStream {
    let fields: Vec<_> = fields.map(ComponentField::from).collect();
    let field_names: Vec<_> = fields.iter().map(|f| f.ident).collect();
    let first_from_world = first_from_world_method(&field_names, &fields);
    let from_world = from_world_method(&field_names, &fields);
    let create = create_method(&fields);
    let update = update_method(&field_names, &fields);
    quote! {
        impl #generics component_group::ComponentGroup for #ident #generics {
            #first_from_world
            #from_world
            #create
            #update
        }
    }
}

fn first_from_world_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let joinables = fields.into_iter().map(|f| f.joinable());
    let clones = fields.into_iter().map(|f| f.cloned());
    let tys = fields.into_iter().map(|f| f.ty);
    quote! {
        fn first_from_world(world: &specs::World) -> Option<Self> {
            let ( #(#field_names),* ) = world.system_data::<( #(specs::ReadStorage<#tys>),* )>();
            ( #(#joinables),* ).join().next().map(|( #(#field_names),* )| Self {
                #(#field_names : #clones),*
            })
        }
    }
}

fn from_world_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let tys = fields.into_iter().map(|f| f.ty);
    let reads = fields.into_iter().map(|f| f.read_value());
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
    let builder_adds = fields.into_iter().map(|f| f.add_to_builder());
    quote! {
        fn create(self, world: &mut specs::World) -> specs::Entity {
            use specs::Builder;
            #[allow(unused_mut)]
            let mut builder = world.create_entity();
            #( #builder_adds )*
            builder.build()
        }
    }
}

fn update_method(field_names: &[&Ident], fields: &[ComponentField]) -> TokenStream {
    let tys = fields.into_iter().map(|f| f.ty);
    let updates = fields.into_iter().map(|f| f.update_value());
    quote! {
        type UpdateError = specs::error::Error;
        fn update(self, entity: specs::Entity, world: &mut specs::World) -> Result<(), Self::UpdateError> {
            let ( #(mut #field_names),* ) = world.system_data::<( #( specs::WriteStorage<#tys> ),* )>();

            #( #updates )*

            Ok(())
        }
    }
}
