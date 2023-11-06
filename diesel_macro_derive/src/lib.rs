#![recursion_limit = "1024"]

extern crate heck;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use heck::SnakeCase;
use proc_macro2::Span;
use syn::{Data, DeriveInput, Fields, Ident, LitByteStr, Type};

#[proc_macro_derive(DieselTypes)]
pub fn derive_diesel_types(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let type_name = format!("diesel_impls_for_{}", name);

    let mod_name = Ident::new(type_name.to_lowercase().as_ref(), Span::call_site());

    let diesel_impls = get_diesel_impls(&input.data, &name);

    let expanded = quote! {
        mod #mod_name {
            #![allow(unused_imports)]

            use std::error::Error;
            use std::io::Write;
            use std::str;

            use diesel::deserialize::FromSql;
            use diesel::expression::bound::Bound;
            use diesel::expression::AsExpression;
            use diesel::pg::Pg;
            use diesel::row::Row;
            use diesel::serialize::Output;
            use diesel::types::{FromSqlRow, IsNull, ToSql};
            use diesel::sql_types::*;
            use diesel::Queryable;
            use diesel::backend::Backend;

            use super::#name;

            #diesel_impls

        }
    };

    // Hand the output tokens back to the compiler.
    expanded.into()
}

fn match_types_names_to_diesel_types(type_name: &str) -> Option<proc_macro2::TokenStream> {
    match type_name.to_lowercase() {
        ref x if x == "uuid" => Some(quote! {::diesel::sql_types::Uuid}),
        ref x if x == "i32" => Some(quote! {::diesel::sql_types::Integer}),
        ref x if x == "i64" => Some(quote! {::diesel::sql_types::BigInt}),
        ref x if x == "string" => Some(quote! {::diesel::sql_types::VarChar}),
        ref x if x == "f64" => Some(quote! {::diesel::sql_types::Double}),
        _ => None,
    }
}

fn get_diesel_impls(data: &Data, name: &Ident) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    fields
                        .unnamed
                        .iter()
                        .map(|f| {
                            let diesel_type = match &f.ty {
                                Type::Path(p) => p
                                    .path
                                    .segments
                                    .iter()
                                    .filter_map(|segment| match_types_names_to_diesel_types(&segment.ident.to_string()))
                                    .nth(0)
                                    .unwrap(),
                                _ => unimplemented!(),
                            };
                            quote! {
                                impl<'a> AsExpression<#diesel_type> for &'a #name {
                                    type Expression = Bound<#diesel_type, &'a #name>;

                                    fn as_expression(self) -> Self::Expression {
                                        Bound::new(self)
                                    }
                                }

                                impl AsExpression<#diesel_type> for #name {
                                    type Expression = Bound<#diesel_type, #name>;

                                    fn as_expression(self) -> Self::Expression {
                                        Bound::new(self)
                                    }
                                }

                                impl ToSql<#diesel_type, Pg> for #name {
                                    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
                                        ToSql::<#diesel_type, Pg>::to_sql(&self.0, out)
                                    }
                                }

                                impl FromSqlRow<#diesel_type, Pg> for #name {
                                    fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
                                        FromSql::<#diesel_type, Pg>::from_sql(row.take()).map(#name)
                                    }
                                }

                                impl FromSql<#diesel_type, Pg> for #name
                                {
                                    fn from_sql(raw: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
                                        FromSql::<#diesel_type, Pg>::from_sql(raw).map(#name)
                                    }
                                }

                                impl Queryable<#diesel_type, Pg> for #name {
                                    type Row = Self;

                                    fn build(row: Self::Row) -> Self {
                                        row
                                    }
                                }

                                impl<'a> AsExpression<Nullable<#diesel_type>> for &'a #name {
                                    type Expression = Bound<Nullable<#diesel_type>, &'a #name>;

                                    fn as_expression(self) -> Self::Expression {
                                        Bound::new(self)
                                    }
                                }

                                impl AsExpression<Nullable<#diesel_type>> for #name {
                                    type Expression = Bound<Nullable<#diesel_type>, #name>;

                                    fn as_expression(self) -> Self::Expression {
                                        Bound::new(self)
                                    }
                                }

                                impl ToSql<Nullable<#diesel_type>, Pg> for #name {
                                    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
                                        ToSql::<Nullable<#diesel_type>, Pg>::to_sql(&self.0, out)
                                    }
                                }
                            }
                        })
                        .nth(0)
                        .unwrap()
                }
                _ => unimplemented!(),
            }
        }
        Data::Enum(ref data) => {
            let variant_ids: Vec<proc_macro2::TokenStream> = data
                .variants
                .iter()
                .map(|variant| {
                    if let Fields::Unit = variant.fields {
                        let id = &variant.ident;
                        quote! {
                            #name::#id
                        }
                    } else {
                        panic!("Variants must be fieldless")
                    }
                })
                .collect();
            let variants_db: Vec<LitByteStr> = data
                .variants
                .iter()
                .map(|variant| LitByteStr::new(variant.ident.to_string().to_snake_case().as_bytes(), Span::call_site()))
                .collect();
            let variants_rs: &[proc_macro2::TokenStream] = &variant_ids;
            let variants_db: &[LitByteStr] = &variants_db;
            quote! {
                impl NotNull for #name {}
                impl SingleValue for #name {}
                impl Queryable<VarChar, Pg> for #name {
                    type Row = #name;
                    fn build(row: Self::Row) -> Self {
                        row
                    }
                }
                impl AsExpression<VarChar> for #name {
                    type Expression = Bound<VarChar, #name>;
                    fn as_expression(self) -> Self::Expression {
                        Bound::new(self)
                    }
                }
                impl<'a> AsExpression<VarChar> for &'a #name {
                    type Expression = Bound<VarChar, &'a #name>;
                    fn as_expression(self) -> Self::Expression {
                        Bound::new(self)
                    }
                }
                impl<'a> AsExpression<Nullable<VarChar>> for &'a #name {
                    type Expression = Bound<Nullable<VarChar>, &'a #name>;

                    fn as_expression(self) -> Self::Expression {
                        Bound::new(self)
                    }
                }

                impl AsExpression<Nullable<VarChar>> for #name {
                    type Expression = Bound<Nullable<VarChar>, #name>;

                    fn as_expression(self) -> Self::Expression {
                        Bound::new(self)
                    }
                }
                impl ToSql<VarChar, Pg> for #name {
                    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
                        match *self {
                            #(#variants_rs => out.write_all(#variants_db)?,)*
                        }
                        Ok(IsNull::No)
                    }
                }
                impl ToSql<Nullable<VarChar>, Pg> for #name {
                    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
                        match *self {
                            #(#variants_rs => out.write_all(#variants_db)?,)*
                        }
                        Ok(IsNull::No)
                    }
                }
                impl FromSqlRow<VarChar, Pg> for #name {
                    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
                        match row.take() {
                            #(Some(#variants_db) => Ok(#variants_rs),)*
                            Some(v) => Err(format!("Unrecognized enum variant: {:?}", str::from_utf8(v).unwrap_or("unreadable value")).to_string().into()),
                            None => Err("Unexpected null for non-null column".into()),
                        }
                    }
                }
                impl FromSql<VarChar, Pg> for #name
                {
                    fn from_sql(raw: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
                        match raw {
                            #(Some(#variants_db) => Ok(#variants_rs),)*
                            Some(v) => Err(format!("Unrecognized enum variant: {:?}", str::from_utf8(v).unwrap_or("unreadable value")).to_string().into()),
                            None => Err("Unexpected null for non-null column".into()),
                        }
                    }
                }
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}
