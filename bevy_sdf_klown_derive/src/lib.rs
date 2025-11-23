use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

fn type_bits(ty: &Type) -> usize {
    match ty {
        Type::Path(p) => {
            let ident = p.path.segments.last().unwrap().ident.to_string();
            match ident.as_str() {
                "f32" => 32,
                "Vec2" => 64,
                "Vec3" => 96,
                "Vec4" => 128,
                _ => 32, // default fallback
            }
        }
        _ => 32,
    }
}

#[proc_macro_derive(EnumVariantGpuFields)]
pub fn enum_variant_gpu_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let match_arms = match &input.data {
        Data::Enum(data_enum) => data_enum.variants.iter().map(|v| {
            let var_name = &v.ident;

            let field_count_expr = match &v.fields {
                Fields::Named(fields) => {
                    let counts = fields.named.iter().map(|f| {
                        let bits = type_bits(&f.ty);
                        quote! { ((#bits + 31)/32) } // round up
                    });
                    quote! { 0 #( + #counts )* }
                }
                Fields::Unnamed(fields) => {
                    let counts = fields.unnamed.iter().map(|f| {
                        let bits = type_bits(&f.ty);
                        quote! { ((#bits + 31)/32) }
                    });
                    quote! { 0 #( + #counts )* }
                }
                Fields::Unit => quote! { 0 },
            };

            let arm = match &v.fields {
                Fields::Named(_) => quote! { Self::#var_name { .. } => #field_count_expr },
                Fields::Unnamed(_) => quote! { Self::#var_name(..) => #field_count_expr },
                Fields::Unit => quote! { Self::#var_name => 0 },
            };

            arm
        }),
        _ => panic!("EnumVariantGpuFields can only be derived for enums"),
    };

    let expanded = quote! {
        impl #name {
            pub fn gpu_field_count(&self) -> usize {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
