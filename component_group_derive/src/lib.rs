extern crate proc_macro;

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

#[proc_macro_derive(ComponentGroup)]
pub fn derive_component_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let DeriveInput {ident, generics, data, ..} = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(DataStruct {fields: Fields::Named(fields), ..}) => {
            impl_component_group(ident, &generics, &fields).into()
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
fn impl_component_group(ident: Ident, generics: &Generics, fields: &FieldsNamed) -> TokenStream {
    let first_from_world = first_from_world_method(&fields);
    let from_world = from_world_method(&fields);
    let create = create_method(&fields);
    let update = update_method(&fields);
    quote! {
        impl #generics component_group::ComponentGroup for #ident #generics {
            #first_from_world
            #from_world
            #create
            #update
        }
    }
}

fn first_from_world_method(fields: &FieldsNamed) -> TokenStream {
    let field_names = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let field_names2 = field_names; // Needed to work around limitations of quote
    let tys = fields.named.iter().map(|f| &f.ty);
    quote! {
        fn first_from_world(world: &specs::World) -> Option<Self> {
            let ( #(#field_names),* ) = world.system_data::<( #(specs::ReadStorage<#tys>),* )>();
            ( #( & #field_names),* ).join().next().map(|( #(#field_names),* )| Self {
                #(#field_names : Clone::clone(#field_names2)),*
            })
        }
    }
}

fn from_world_method(fields: &FieldsNamed) -> TokenStream {
    let field_names = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let field_names2 = field_names; // Needed to work around limitations of quote
    let tys = fields.named.iter().map(|f| &f.ty);
    let err_msgs = fields.named.iter()
        .map(|Field {ty, ..}| format!("bug: expected a {} component to be present", quote!(#ty)));
    quote! {
        fn from_world(entity: specs::Entity, world: &specs::World) -> Self {
            let ( #(#field_names),* ) = world.system_data::<( #(specs::ReadStorage<#tys>),* )>();

            Self {
                #(
                    #field_names : #field_names2.get(entity).cloned().expect(#err_msgs)
                ),*
            }
        }
    }
}

fn create_method(fields: &FieldsNamed) -> TokenStream {
    let field_names = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    quote! {
        fn create(self, world: &mut specs::World) -> specs::Entity {
            use specs::Builder;
            world.create_entity()
                #( .with(self.#field_names) )*
                .build()
        }
    }
}

fn update_method(fields: &FieldsNamed) -> TokenStream {
    let field_names = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let field_names2 = field_names; // Needed to work around limitations of quote
    let tys = fields.named.iter().map(|f| &f.ty);
    quote! {
        type UpdateError = specs::error::Error;
        fn update(self, entity: specs::Entity, world: &mut specs::World) -> Result<(), Self::UpdateError> {
            let ( #(mut #field_names),* ) = world.system_data::<( #( specs::WriteStorage<#tys> ),* )>();

            #( #field_names.insert(entity, self.#field_names2)?; )*

            Ok(())
        }
    }
}
