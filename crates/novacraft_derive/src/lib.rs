use proc_macro2::TokenStream;
use quote::*;
use spanned::Spanned;
use syn::{self, DeriveInput};

#[proc_macro_derive(InitBlocks)]
pub fn init_blocks_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    let (blocks_enum, into_str, impl_debug) = blocks_enum(&ast);
    let default_registries = def_registries(&ast);

    let a = quote! {
        // pub mod auto_generated {
            #blocks_enum
            #into_str
            #impl_debug
            #default_registries
        // }
    };

    a.into()
}

fn blocks_enum(input: &DeriveInput) -> (TokenStream, TokenStream, TokenStream) {
    let syn::Data::Enum(syn::DataEnum { variants, .. }) = &input.data else {
        panic!("Init Blocks macro is only available to enums");
    };

    let vidents: Vec<syn::Ident> = variants.iter().map(|var| var.ident.clone()).collect();
    let capitalized_vidents: Vec<syn::Ident> = vidents
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_uppercase().as_str(), ident.span()))
        .collect();
    let vnames: Vec<String> = vidents.iter().map(|ident| format!("{}", ident)).collect();
    let id_vals = 0u16..(vidents.len() as u16);
    let enum_name: syn::Ident = syn::Ident::new_raw("Block", vidents.clone().last().__span());

    let blocks_enum = quote! {
        #[repr(u16)]
        #[derive(Eq, PartialEq, Clone, Copy, bevy::prelude::Component)]
        pub enum #enum_name {
            #(#capitalized_vidents = #id_vals),*
        }
    };

    let impl_into_str = quote! {
        impl Into<&'static str> for #enum_name {
            fn into(self) -> &'static str {
                match self {
                    #(Self::#capitalized_vidents => #vnames),*
                }
            }
        }
    };

    let impl_debug = quote! {
        impl std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "NovaCraft::Block::{}", Into::<&'static str>::into(*self))
            }
        }
    };

    (blocks_enum.into(), impl_into_str.into(), impl_debug.into())
}

fn def_registries(input: &DeriveInput) -> TokenStream {
    let syn::Data::Enum(syn::DataEnum { variants, .. }) = &input.data else {
        panic!("Init Blocks macro is only available to enums");
    };
    let vidents: Vec<syn::Ident> = variants.iter().map(|var| var.ident.clone()).collect();
    let lowercase_vidents: Vec<syn::Ident> = vidents
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_lowercase().as_str(), ident.span()))
        .collect();
    let fields: Vec<&syn::Field> = variants
        .iter()
        .map(|var| {
            let syn::Fields::Unnamed(ref f) = var.fields else {
                panic!("a")
            };
            f.unnamed.first().expect("aa")
        })
        .collect();
    let fpaths: Vec<syn::Path> = fields
        .iter()
        .map(|&f| {
            let syn::Type::Path(ref path) = f.ty else {
                panic!("f");
            };
            path.path.clone()
        })
        .collect();

    let impl_default = quote! {
        impl Default for BlockPropertyRegistry<PhysicalProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().physical_properties),*
                }
            }
        }
        impl Default for BlockPropertyRegistry<PassiveProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().passive_properties),*
                }
            }
        }
        impl Default for BlockPropertyRegistry<PerceptibleProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().perceptible_properties),*
                }
            }
        }
    };

    impl_default.into()
}