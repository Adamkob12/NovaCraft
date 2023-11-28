use proc_macro2::TokenStream;
use quote::*;
use spanned::Spanned;
use syn::{self, DeriveInput};

#[proc_macro_derive(InitBlockProperties)]
pub fn init_block_properties(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: &DeriveInput = &syn::parse(input).unwrap();
    let syn::Data::Enum(syn::DataEnum { variants, .. }) = &ast.data else {
        panic!("Init Blocks macro is only available to enums");
    };
    let vidents: Vec<syn::Ident> = variants.iter().map(|var| var.ident.clone()).collect();
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
    let props: Vec<syn::Path> = fpaths
        .iter()
        .map(|p| {
            let syn::Path { segments, .. } = p;
            let seg = segments.first().unwrap();
            let syn::PathSegment {
                arguments:
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args, ..
                    }),
                ..
            } = seg
            else {
                panic!()
            };
            let syn::GenericArgument::Type(syn::Type::Path(path)) = args.first().unwrap() else {
                panic!()
            };
            path.path.clone()
        })
        .collect();

    {
        quote! {
            #(impl BlockProperty for #props {
                fn get_property_type() -> BlockPropertyTypes {
                     BlockPropertyTypes::#vidents(None)
                }
            })*
        }
    }
    .into()
}

#[proc_macro_derive(InitBlocks)]
pub fn init_blocks_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    let (blocks_enum, into_str, impl_debug, enum_name) = blocks_enum(&ast);
    let default_registries = def_registries(&ast, enum_name);

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

fn blocks_enum(input: &DeriveInput) -> (TokenStream, TokenStream, TokenStream, syn::Ident) {
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

    let blocks_enum = quote! { #[repr(u16)]
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

    (
        blocks_enum.into(),
        impl_into_str.into(),
        impl_debug.into(),
        enum_name,
    )
}

fn def_registries(input: &DeriveInput, enum_name: syn::Ident) -> TokenStream {
    let syn::Data::Enum(syn::DataEnum { variants, .. }) = &input.data else {
        panic!("Init Blocks macro is only available to enums");
    };
    let vidents: Vec<syn::Ident> = variants.iter().map(|var| var.ident.clone()).collect();
    let lowercase_vidents: Vec<syn::Ident> = vidents
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_lowercase().as_str(), ident.span()))
        .collect();
    let capitalized_vidents: Vec<syn::Ident> = vidents
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_uppercase().as_str(), ident.span()))
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
    let props: Vec<Vec<syn::Field>> = variants
        .iter()
        .map(|var| {
            let syn::Fields::Unnamed(ref f) = var.fields else {
                panic!("a")
            };
            f.unnamed.clone().iter().skip(1).cloned().collect()
        })
        .collect();
    let props = props.first().unwrap();
    let props: Vec<syn::Path> = props
        .iter()
        .map(|f| {
            let syn::Type::Path(ref path) = f.ty else {
                panic!()
            };
            path.path.clone()
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
        use crate::blocks::existence_conditions::*;
        use crate::blocks::properties::*;
        #[derive(bevy::prelude::Resource)]
        pub struct BlockPropertyRegistry<P: BlockProperty> {
            #(pub #lowercase_vidents: PropertyCollection<P>),*
        }

        // #(impl Default for BlockPropertyRegistry<#props> {
        //     fn default() -> Self {
        //         Self {
        //             #(#lowercase_vidents: #fpaths::#vidents())::#props,*
        //         }
        //     }
        // });*

        impl Default for BlockPropertyRegistry<PhysicalProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().PhysicalPropertys),*
                }
            }
        }
        impl Default for BlockPropertyRegistry<PassiveProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().PassivePropertys),*
                }
            }
        }
        impl Default for BlockPropertyRegistry<PerceptibleProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().PerceptiblePropertys),*
                }
            }
        }
        impl Default for BlockPropertyRegistry<ExistenceCondition> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().ExistenceConditions),*
                }
            }
        }
        impl Default for BlockPropertyRegistry<DynamicProperty> {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().DynamicPropertys),*
                }
            }
        }


        impl<P: BlockProperty> BlockPropertyRegistry<P> {
            pub fn get_properties(&self, block: &#enum_name) -> &[P] {
                match block {
                    #(#enum_name::#capitalized_vidents => self.#lowercase_vidents.0.as_slice()),*
                }
            }
        }

        impl<P: BlockProperty + PartialEq> BlockPropertyRegistry<P> {
            pub fn contains_property(&self, block: &#enum_name, property: &P) -> bool {
                match block {
                    #(#enum_name::#capitalized_vidents => self.#lowercase_vidents.contains(property)),*
                }
            }
        }
    };

    impl_default.into()
}
