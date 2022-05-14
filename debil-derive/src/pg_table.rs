use proc_macro2::*;
use quote::quote;
use syn::{Meta, Result};

struct FieldAttribute {
    primary_key: bool,
    size: Option<usize>,
    other: String,
}

pub fn impl_pg_table(ast: syn::DeriveInput) -> Result<TokenStream> {
    let name = &ast.ident;

    let schema = match ast.data {
        syn::Data::Struct(ds) => ds.fields,
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    };

    // Only one primary key is allowed for now:
    let mut has_primary_key = false;

    let columns = schema
        .into_iter()
        .map(|f| {
            let ident = f.ident.unwrap();
            let ty = f.ty;
            let mut field_attr = FieldAttribute {
                primary_key: false,
                size: None,
                other: String::new(),
            };

            for attr in f.attrs {
                match attr.parse_meta().unwrap() {
                    Meta::List(l) if l.path.is_ident("sql") => {
                        for meta in l.nested {
                            match meta {
                                syn::NestedMeta::Meta(Meta::NameValue(nv)) => {
                                    if nv.path.is_ident("size") {
                                        match nv.lit {
                                            syn::Lit::Int(i) => {
                                                let size = i.base10_parse::<usize>().unwrap();

                                                field_attr.size = Some(size);
                                            }
                                            _ => todo!(),
                                        }
                                    } else if nv.path.is_ident("primary_key") {
                                        field_attr.primary_key = true;
                                    } else {
                                        todo!()
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }

            let field_attr_quote = {
                let primary_key = field_attr.primary_key;
                if primary_key {
                    if has_primary_key {
                        panic!("Only one primary key is allowed for now");
                    }
                    has_primary_key = true;
                }

                let size = field_attr
                    .size
                    .map(|s| quote! { Some(#s) })
                    .unwrap_or(quote! { None });
                let other = field_attr.other;

                quote! {
                    FieldAttribute {
                        primary_key: #primary_key,
                        size: #size,
                        other: #other.to_string(),
                    }
                }
            };

            quote! {
                (
                    stringify!(#ident),
                    PgTypeSelector::select_type(std::marker::PhantomData::<#ty>),
                    #field_attr_quote,
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
