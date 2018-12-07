extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input,
    DeriveInput,
    Data,
    DataStruct,
    DataEnum,
    DataUnion,
    token::{Enum, Union},
    Fields,
};
use quote::quote;

#[proc_macro_derive(ComponentGroup)]
pub fn derive_component_group(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let DeriveInput {attrs, vis, ident, generics, data} = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(DataStruct {struct_token, fields, ..}) => match fields {
            Fields::Named(fields) => TokenStream::from(quote! {

            }),
            _ => {
                syn::Error::new(struct_token.span, "Only structs with named fields are supported")
                    .to_compile_error()
                    .into()
            },
        },
        Data::Enum(DataEnum {enum_token: Enum {span}, ..}) |
        Data::Union(DataUnion {union_token: Union {span}, ..}) => {
            syn::Error::new(span, "Only structs with named fields are supported")
                .to_compile_error()
                .into()
        },
    }
}
