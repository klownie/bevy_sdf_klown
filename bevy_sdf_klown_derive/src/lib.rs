use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Type, parse_macro_input};

// Convert a field into a vec of f32, expanding Vec2/Vec3/Vec4
fn flatten_expr(ident: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(p) => {
            let ident_str = p.path.segments.last().unwrap().ident.to_string();
            match ident_str.as_str() {
                "f32" => quote! { vec![#ident] },
                "Vec2" => quote! { vec![#ident.x, #ident.y] },
                "Vec3" => quote! { vec![#ident.x, #ident.y, #ident.z] },
                "Vec4" => quote! { vec![#ident.x, #ident.y, #ident.z, #ident.w] },
                _ => quote! { vec![#ident as f32] },
            }
        }
        _ => quote! { vec![#ident as f32] },
    }
}

#[proc_macro_derive(EnumVariantGpuFields)]
pub fn enum_variant_gpu_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let data_enum = match &input.data {
        Data::Enum(data) => data,
        _ => panic!("EnumVariantGpuFields can only be derived for enums"),
    };

    // Generate gpu_field_count() arms
    let count_arms = data_enum.variants.iter().map(|v| {
        let var_name = &v.ident;
        let field_count_expr = match &v.fields {
            Fields::Named(fields) => {
                let counts = fields.named.iter().map(|f| {
                    let bits = match &f.ty {
                        Type::Path(p) => {
                            match p.path.segments.last().unwrap().ident.to_string().as_str() {
                                "f32" => 32,
                                "Vec2" => 64,
                                "Vec3" => 96,
                                "Vec4" => 128,
                                _ => 32,
                            }
                        }
                        _ => 32,
                    };
                    quote! { ((#bits + 31)/32) as usize }
                });
                quote! { 0usize #( + #counts )* }
            }
            _ => panic!("All variants must be struct variants with named fields"),
        };
        quote! { Self::#var_name { .. } => #field_count_expr }
    });

    // Generate flatten_fields() arms
    let flatten_arms = data_enum.variants.iter().map(|v| {
        let var_name = &v.ident;
        let fields_named = match &v.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect::<Vec<_>>(),
            _ => panic!("All variants must be struct variants with named fields"),
        };

        // Generate expressions for each field
        let exprs = fields_named.iter().map(|f| {
            flatten_expr(
                f,
                &v.fields
                    .iter()
                    .find(|ff| ff.ident.as_ref() == Some(f))
                    .unwrap()
                    .ty,
            )
        });

        // Generate match arm with field bindings
        quote! {
            Self::#var_name { #(#fields_named),* } => {
                let mut v = Vec::new();
                #(v.extend(#exprs);)*
                v
            }
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn gpu_field_count(&self) -> usize {
                match self {
                    #(#count_arms),*
                }
            }

            pub fn flatten_fields(&self) -> Vec<f32> {
                match self {
                    #(#flatten_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
