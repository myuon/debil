use proc_macro2::*;
use quote::quote;
use syn::{Meta, Result};

pub fn impl_pg_table(ast: syn::DeriveInput) -> Result<TokenStream> {
    let name = &ast.ident;

    let schema = match ast.data {
        syn::Data::Struct(ds) => ds.fields,
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };
    let columns = schema
        .into_iter()
        .map(|f| {
            let ident = f.ident.unwrap();
            let ty = f.ty;
            let attr_size_usize = f.attrs.iter().find_map(|a| match a.parse_meta().ok() {
                Some(Meta::List(l)) if l.path.is_ident("sql") => {
                    l.nested.into_iter().find_map(|n| match n {
                        syn::NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("size") => {
                            match nv.lit {
                                syn::Lit::Int(i) => i.base10_parse::<usize>().ok(),
                                _ => None,
                            }
                        }
                        _ => todo!(),
                    })
                }
                _ => None,
            });
            let attr_size_code = attr_size_usize
                .map(|u| quote! { Some(#u) })
                .unwrap_or(quote! { None });

            quote! {
                (
                    stringify!(#ident),
                    PgTypeSelector::select_type(std::marker::PhantomData::<#ty>),
                    FieldAttribute {
                        size: #attr_size_code,
                        other: "".to_string(),
                    },
                ),
            }
        })
        .collect::<Vec<_>>();

    let gen = quote! {
        impl PgTable for #name {
            fn table_name(_: std::marker::PhantomData<Self>) -> &'static str {
                stringify!(#name)
            }

            fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(&'static str, PgType, FieldAttribute)> {
                vec![
                    #(#columns)*
                ]
            }
        }
    };

    Ok(gen.into())
}

pub fn impl_from_row(ast: syn::DeriveInput) -> Result<TokenStream> {
    let name = &ast.ident;

    let schema = match ast.data {
        syn::Data::Struct(ds) => ds.fields,
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };

    let fields = schema
        .into_iter()
        .map(|field| {
            let label = field.ident.unwrap();

            quote! {
                #label: row.try_get(stringify!(#label))?,
            }
        })
        .collect::<Vec<_>>();

    let gen = quote! {
        impl<'r> FromRow<'r, PgRow> for Test {
            fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
                Ok(#name {
                    #(#fields)*
                })
            }
        }
    };

    Ok(gen.into())
}

pub fn impl_binds(ast: syn::DeriveInput) -> Result<TokenStream> {
    let name = &ast.ident;

    let schema = match ast.data {
        syn::Data::Struct(ds) => ds.fields,
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };
    let fields = schema
        .into_iter()
        .map(|field| {
            let label = field.ident.unwrap();

            quote! {
                #label,
            }
        })
        .collect::<Vec<_>>();

    let binds_macro_name = Ident::new(&format!("binds_{}", name), Span::call_site());
    let binds_cond_macro_name = Ident::new(&format!("binds_cond_{}", name), Span::call_site());

    let gen = quote! {
        macro_rules! #binds_macro_name {
            ($q:expr,$e:expr $(,)?) => {
                binds!(
                    $q,
                    $e,
                    #(#fields)*
                )
            };
        }

        macro_rules! #binds_cond_macro_name {
            ($q:expr,$e:expr,$ns:expr,$($name:ident),* $(,)?) => {{
                binds_cond!(
                    $q,
                    $e,
                    $ns,
                    #(#fields)*
                )
            }};
        }
    };

    Ok(gen.into())
}
