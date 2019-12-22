extern crate proc_macro;

use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Result};

struct TableAttr {
    table_name: String,
    primary_key_columns: Vec<String>,
    sql_type: proc_macro2::TokenStream,
}

struct AttrInput {
    paren_token: syn::token::Paren,
    attrs: syn::punctuated::Punctuated<KeyValue, syn::Token![,]>,
}

impl Parse for AttrInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(AttrInput {
            paren_token: syn::parenthesized!(content in input),
            attrs: content.parse_terminated(KeyValue::parse)?,
        })
    }
}

impl AttrInput {
    fn to_table_attr(self, table_name: String) -> TableAttr {
        let mut table = TableAttr {
            table_name: table_name,
            primary_key_columns: vec![],
            sql_type: quote! { Vec<u8> },
        };

        for attr in self.attrs.into_iter() {
            match format!("{}", attr.key).as_str() {
                "table_name" => table.table_name = attr.value.as_str().unwrap(),
                "sql_type" => {
                    let sql_type =
                        syn::parse_str::<syn::Type>(&attr.value.as_str().unwrap()).unwrap();
                    table.sql_type = quote! { #sql_type };
                }
                "primary_key_columns" => {
                    table.primary_key_columns = attr.value.as_str().unwrap().split(",").map(|s| s.to_string()).collect();
                },
                d => panic!("unsupported attribute: {}", d),
            }
        }

        table
    }

    fn to_attr_map(self) -> HashMap<String, Universe> {
        let mut result = HashMap::new();

        for attr in self.attrs.into_iter() {
            result.insert(format!("{}", attr.key), attr.value);
        }

        result
    }
}

#[derive(Clone)]
enum Universe {
    VStr(String),
    VI32(i32),
    VBool(bool),
    VType(syn::Type),
}

impl Parse for Universe {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitStr) {
            input
                .parse::<syn::LitStr>()
                .map(|v| Universe::VStr(v.value()))
        } else if lookahead.peek(syn::LitInt) {
            input
                .parse::<syn::LitInt>()
                .map(|v| Universe::VI32(v.base10_parse::<i32>().unwrap()))
        } else if lookahead.peek(syn::LitBool) {
            input
                .parse::<syn::LitBool>()
                .map(|v| Universe::VBool(v.value))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Universe {
    fn as_str(self) -> Option<String> {
        use Universe::*;
        match self {
            VStr(s) => Some(s),
            _ => None,
        }
    }

    fn as_i32(self) -> Option<i32> {
        use Universe::*;
        match self {
            VI32(i) => Some(i),
            _ => None,
        }
    }

    fn as_bool(self) -> Option<bool> {
        use Universe::*;
        match self {
            VBool(b) => Some(b),
            _ => None,
        }
    }
}

struct KeyValue {
    key: proc_macro2::Ident,
    punct: syn::Token![=],
    value: Universe,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(KeyValue {
            key: input.parse()?,
            punct: input.parse()?,
            value: input.parse()?,
        })
    }
}

fn get_fields_from_datastruct(
    data: syn::Data,
) -> Vec<(proc_macro2::Ident, syn::Type, HashMap<String, Universe>)> {
    let mut result = Vec::new();

    match data {
        syn::Data::Struct(st) => match st.fields {
            syn::Fields::Named(fields) => {
                for name in fields.named.iter() {
                    result.push((
                        name.ident.as_ref().unwrap().clone(),
                        name.ty.clone(),
                        if name.attrs.len() == 0 {
                            HashMap::new()
                        } else {
                            // TODO: Only first FieldAttr will be effective
                            syn::parse2::<AttrInput>(name.attrs[0].tokens.clone())
                                .unwrap()
                                .to_attr_map()
                        },
                    ));
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }

    result
}

fn option_to_quote<T: quote::ToTokens>(opt: Option<T>) -> proc_macro2::TokenStream {
    if opt.is_some() {
        quote! { Some(#opt) }
    } else {
        quote! { None }
    }
}

#[proc_macro_derive(Table, attributes(sql))]
pub fn derive_record(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    if input.attrs.is_empty() {
        panic!("Currently, sql(table_name),sql(sql_type) and primary_key_columns(comma_separated_string) are required.");
    }

    let attr_stream = input.attrs[0].tokens.clone();
    let table_attr = syn::parse2::<AttrInput>(attr_stream)
        .unwrap()
        .to_table_attr(format!("{}", ident));
    let table_name = table_attr.table_name;

    let field_struct = get_fields_from_datastruct(input.data);

    let primary_key_columns = table_attr.primary_key_columns;
    if primary_key_columns.len() == 0 {
        panic!("At least one primary key must be specified")
    }
    
    // checking existince of keys specified as primary key
    for pk_column_name in primary_key_columns.iter() {
            field_struct.iter().find(|(ident,_,_)|
                ident == pk_column_name
            ).expect(&format!("primary_key: {} was not found in this table struct",pk_column_name));
    };

    let push_primary_key_columns = primary_key_columns.iter()
    .map(|v| 
        quote! { result.push(#v.to_string()); }    
    )
    .collect::<Vec<_>>();
    
    let push_field_names = field_struct
        .iter()
        .map(|(ident, _, _)| quote! { result.push((stringify!(#ident).to_string(), SQLValue::serialize(self.#ident))); })
        .collect::<Vec<_>>();
    let push_column_schema = field_struct
        .iter()
        .map(move |(ident, ty, attr_map)| {
            let size_opt = attr_map.get("size").map(|v| v.clone().as_i32().unwrap());
            let size = option_to_quote(size_opt);
            let unique = option_to_quote(attr_map.get("unique").map(|v| v.clone().as_bool().unwrap()));
            let not_null = option_to_quote(attr_map.get("not_null").map(|v| v.clone().as_bool().unwrap()));
            let size_unopt = size_opt.unwrap_or(0);

            quote! {
                result.push((stringify!(#ident).to_string(), <Self::ValueType as SQLValue<_>>::column_type(std::marker::PhantomData::<#ty>, #size_unopt), FieldAttribute {
                    size: #size,
                    unique: #unique,
                    not_null: #not_null,
                }));
            }
        })
        .collect::<Vec<_>>();
    let record_fields = field_struct
        .iter()
        .map(|(ident, _, _)| {
            quote! {
                #ident: <Self::ValueType as SQLValue<_>>::deserialize(values.get(stringify!(#ident)).unwrap().clone()),
            }
        })
        .collect::<Vec<_>>();

    let sql_type = table_attr.sql_type;

    let expanded = quote! {
        impl SQLMapper for #ident {
            type ValueType = #sql_type;

            fn map_from_sql(values: std::collections::HashMap<String, Self::ValueType>) -> Self {
                #ident {
                    #( #record_fields )*
                }
            }
        }

        impl SQLTable for #ident {
            fn table_name(_: std::marker::PhantomData<Self>) -> String {
                #table_name.to_string()
            }

            fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(String, String, FieldAttribute)> {
                let mut result = Vec::new();
                #( #push_column_schema )*
                result
            }

            fn primary_key_columns(_: std::marker::PhantomData<Self>) -> Vec<String> {
                let mut result = Vec::new();
                #( #push_primary_key_columns )*
                result
            }

            fn map_to_sql(self) -> Vec<(String, Self::ValueType)> {
                let mut result = Vec::new();
                #( #push_field_names )*

                result
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
