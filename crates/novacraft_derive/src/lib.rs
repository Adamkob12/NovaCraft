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
    // let props: Vec<syn::Path> = fpaths
    //     .iter()
    //     .map(|p| {
    //         let syn::Path { segments, .. } = p;
    //         let seg = segments.first().unwrap();
    //         let syn::PathSegment {
    //             arguments:
    //                 syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
    //                     args, ..
    //                 }),
    //             ..
    //         } = seg
    //         else {
    //             panic!()
    //         };
    //         let syn::GenericArgument::Type(syn::Type::Path(path)) = args.first().unwrap() else {
    //             panic!()
    //         };
    //         path.path.clone()
    //     })
    //     .collect();

    {
        quote! {
            #(impl BlockProperty for #fpaths {
            })*
        }
    }
    .into()
}

const CUSTOM_MESH_ATTRIBUTE: &str = "custom_mesh";

#[proc_macro_derive(InitBlocks, attributes(custom_mesh))]
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
        #[derive(Eq, PartialEq, Clone, Copy, bevy::prelude::Component, Hash)]
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
    let custom_mesh_variants: Vec<&syn::Variant> = variants
        .iter()
        .filter(|var| {
            var.attrs
                .iter()
                .any(|attr| attr.path().is_ident(CUSTOM_MESH_ATTRIBUTE))
        })
        .collect();
    let vidents: Vec<syn::Ident> = variants.iter().map(|var| var.ident.clone()).collect();
    let lowercase_vidents: Vec<syn::Ident> = vidents
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_lowercase().as_str(), ident.span()))
        .collect();
    let capitalized_vidents: Vec<syn::Ident> = vidents
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_uppercase().as_str(), ident.span()))
        .collect();

    let cm_vindets: Vec<syn::Ident> = custom_mesh_variants
        .iter()
        .map(|var| var.ident.clone())
        .collect();
    let cm_lowercase_vindents: Vec<syn::Ident> = cm_vindets
        .iter()
        .map(|ident| syn::Ident::new(format!("{}", ident).to_lowercase().as_str(), ident.span()))
        .collect();

    let fields: Vec<&syn::Field> = variants
        .iter()
        .map(|var| {
            if let syn::Fields::Unnamed(ref f) = var.fields {
                f.unnamed.first().expect("aa")
            } else if let syn::Fields::Named(ref f) = var.fields {
                f.named.first().expect("ab")
            } else {
                panic!()
            }
        })
        .collect();

    let props: Vec<Vec<syn::Field>> = variants
        .iter()
        .filter_map(|var| {
            if let syn::Fields::Named(ref f) = var.fields {
                Some(f.named.clone().iter().skip(1).cloned().collect())
            } else {
                None
            }
        })
        .collect();

    let props = props.first().unwrap();
    let props_idents: Vec<syn::Ident> = props.iter().map(|f| f.ident.clone().unwrap()).collect();
    let props_path: Vec<syn::Path> = props
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

    let mut def_impls = vec![];
    for i in 0..props_path.len() {
        let path = props_path[i].clone();
        let ident = props_idents[i].clone();
        def_impls.push(quote! {
            impl Default for BlockPropertyRegistry<#path> {
                fn default() -> Self {
                    Self {
                        #(#lowercase_vidents: #fpaths::#vidents().#ident),*
                    }
                }
            }
        })
    }

    let impl_default = quote! {
        use crate::blocks::existence_conditions::*;
        use crate::blocks::properties::*;
        #[derive(bevy::prelude::Resource)]
        #[repr(C)]
        pub struct BlockPropertyRegistry<P: BlockProperty> {
            #(#lowercase_vidents: PropertyCollection<P>),*
        }

        #(
        #def_impls
        )*

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

    let def_meshreg = quote! {
        use crate::blocks::*;
        #[derive(Resource, Clone)]
        pub struct MeshRegistry {
            #(#lowercase_vidents: VoxelMesh<Mesh>),*
        }

        impl Default for MeshRegistry {
            fn default() -> Self {
                Self {
                    #(#lowercase_vidents: #fpaths::#vidents().mesh_builder.into()),*
                }
            }
        }

        impl VoxelRegistry for MeshRegistry {
            type Voxel = #enum_name;

            fn get_mesh(&self, voxel: &#enum_name) -> VoxelMesh<&Mesh> {
                // println!("getting {:?}", voxel);
                match voxel {
                    #(#enum_name::#capitalized_vidents => self.#lowercase_vidents.ref_mesh()),*
                }
            }

            fn all_attributes(&self) -> Vec<MeshVertexAttribute> {
                vec![
                    Mesh::ATTRIBUTE_POSITION,
                    Mesh::ATTRIBUTE_UV_0,
                    Mesh::ATTRIBUTE_COLOR,
                    Mesh::ATTRIBUTE_NORMAL,
                ]
            }

            fn get_voxel_dimensions(&self) -> [f32; 3] {
                VOXEL_DIMS
            }

            fn get_center(&self) -> [f32; 3] {
                VOXEL_CENTER
            }

            #[allow(unused_variables)]
            fn is_covering(&self, voxel: &#enum_name, side: prelude::Face) -> bool {
                *voxel as u16 != 0 && *voxel != #enum_name::GREENERY
            }
        }
    };

    let args = get_args_of_custom_mesh_attribute_as_token_stream(custom_mesh_variants);
    let registry_plugin = quote! {
        use bevy_asset_loader::prelude::*;
        use bevy::prelude::*;
        pub struct BlockRegistriesPlugin;

        impl Plugin for BlockRegistriesPlugin {
            fn build(&self, app: &mut App) {
                app.add_state::<crate::AssetLoadingState>()
                    .add_loading_state(LoadingState::new(AssetLoadingState::Loading)
                        .continue_to_state(AssetLoadingState::Loaded));

                #(app.init_resource::<BlockPropertyRegistry<#props_path>>();)*
                app.init_resource::<MeshRegistry>();

                app.add_systems(OnEnter(AssetLoadingState::Loaded),
                    put_external_meshes_in_mesh_registry_after_load);

                app.add_collection_to_loading_state::<_, ExternalMeshes>(AssetLoadingState::Loading);
            }
        }

        #[derive(AssetCollection, Resource)]
        struct ExternalMeshes {
            #(
                #[asset(#args)]
                #cm_lowercase_vindents: bevy::prelude::Handle<Mesh>),
            *
        }

        fn put_external_meshes_in_mesh_registry_after_load(
            mut meshes: ResMut<Assets<Mesh>>,
            mut mreg: ResMut<MeshRegistry>,
            loaded_meshes: Res<ExternalMeshes>,
        ) {
            bevy::log::info!("All assets have been loaded.");
            #(mreg.#cm_lowercase_vindents.set(meshes.remove(&loaded_meshes.#cm_lowercase_vindents).unwrap()));*
        }
    };

    let f = quote! {
        #impl_default
        #def_meshreg
        #registry_plugin
    };

    f.into()
}

fn get_args_of_custom_mesh_attribute_as_token_stream(
    variants: Vec<&syn::Variant>,
) -> Vec<TokenStream> {
    let mut args = vec![];
    variants.iter().for_each(|&var| {
        args.push({
            // we can unwrap this, because variants has been filtered already
            let cm_attr = var
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident(CUSTOM_MESH_ATTRIBUTE))
                .unwrap();
            match cm_attr.meta {
                syn::Meta::List(ref list) => list.tokens.clone(),
                _ => {
                    panic!("Expected list-style attribute, as in #[custom_mesh(path = \"path/to/mesh\")]")
                }
            }
        })
    });
    args
}
