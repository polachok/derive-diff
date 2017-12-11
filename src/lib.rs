#![recursion_limit = "128"]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate struct_diff;

use proc_macro::TokenStream;

#[proc_macro_derive(Diff)]
pub fn generate_diff_impl(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_diff(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_diff(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    match ast.body {
        syn::Body::Struct(ref vdata) => impl_diff_struct(name, &vdata),
        syn::Body::Enum(ref variants) => impl_diff_enum(name, &variants),
    }
}

/// Generates Diff for each struct field
struct StructGenerator<'a> {
    fields: &'a [syn::Field]
}

impl<'a> quote::ToTokens for StructGenerator<'a> {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
        for field in self.fields.iter() {
            if let Some(ref field_name) = field.ident {
                let field_name_s = field_name.to_string();
                tokens.append(
                    quote!{
                        if let Some(inner_diffs) = self.#field_name.diff(&other.#field_name) {
                            for diff in inner_diffs {
                                let mut path = String::from(#field_name_s);
                                if !diff.field.is_empty() {
                                    path.push_str(&".");
                                }
                                path.push_str(&diff.field);
                                diffs.push(::struct_diff::Difference {
                                    field: path,
                                    left: diff.left,
                                    right: diff.right,
                                })
                            }
                        }
                    }
                )
            }
        }
    }
}

/// Generates Diff impl for enum fields
struct FieldGenerator<'a> {
    name: &'a syn::Ident,
    fields: &'a [syn::Field],
}

impl<'a> quote::ToTokens for FieldGenerator<'a> {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
        for (i, field) in self.fields.iter().enumerate() {
            let field_name = field.ident.clone().unwrap_or(syn::Ident::from(i));
            let field_name_s = field_name.to_string();
            let left = {
                    let mut new_name = String::from("left_");
                    new_name.push_str(&field_name.to_string());
                    syn::Ident::from(new_name)
            };
            let right = {
                    let mut new_name = String::from("right_");
                    new_name.push_str(&field_name.to_string());
                    syn::Ident::from(new_name)
            };
            let name = self.name.to_string();
            tokens.append(
                quote!{
                    if let Some(inner_diffs) = #left.diff(&#right) {
                        for diff in inner_diffs {
                            let mut path = String::from(#name);
                            path.push_str(".");
                            path.push_str(&#field_name_s);
                            if !diff.field.is_empty() {
                                path.push_str(&".");
                            }
                            path.push_str(&diff.field);
                            diffs.push(::struct_diff::Difference {
                                field: path,
                                left: diff.left,
                                right: diff.right,
                            })
                        }
                    }
                }
            )
        }
    }
}

struct StructPatField<'a> {
    name: &'a syn::Ident,
    prefix: &'static str,
}

impl<'a> quote::ToTokens for StructPatField<'a> {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
            let ident = self.name;
            let prefixed_ident = {
                let mut new_name = String::from(self.prefix);
                new_name.push_str("_");
                new_name.push_str(&ident.to_string());
                syn::Ident::from(new_name)
            };
            tokens.append(quote! {
                #ident: ref #prefixed_ident
            })
    }
}

/// Implements Diff for enum
fn impl_diff_enum(name: &syn::Ident, variants: &[syn::Variant]) -> quote::Tokens {
    let mut differs = Vec::new();
    for variant in variants {
        let var_name = &variant.ident;
        let var_data = &variant.data;
        let diff = match var_data {
            &syn::VariantData::Tuple(ref fields) => {
                let gen = FieldGenerator { name: var_name, fields };
                let names: Vec<syn::Ident> = fields.iter()
                        .enumerate()
                        .map(|(id, field)| field.ident.clone().unwrap_or(syn::Ident::from(id)))
                        .collect();
                let left = names.iter().map(|ident| {
                    let mut new_name = String::from("left_");
                    new_name.push_str(&ident.to_string());
                    syn::Ident::from(new_name)
                }).collect::<Vec<_>>();
                let right = names.iter().map(|ident| {
                    let mut new_name = String::from("right_");
                    new_name.push_str(&ident.to_string());
                    syn::Ident::from(new_name)
                }).collect::<Vec<_>>();
                quote! {
                    (&#name::#var_name(#(ref #left),*), &#name::#var_name(#(ref #right),*)) => {
                       #gen
                    }
                }
            },
            &syn::VariantData::Struct(ref fields) => {
                let gen = FieldGenerator { name: var_name, fields };
                let names: Vec<syn::Ident> = fields.iter()
                        .enumerate()
                        .map(|(id, field)| field.ident.clone().unwrap_or(syn::Ident::from(id)))
                        .collect();
                let left = names.iter().map(|ident| {
                    StructPatField{ name: ident, prefix: "left"}
                }).collect::<Vec<_>>();
                let right = names.iter().map(|ident| {
                    StructPatField{ name: ident, prefix: "right"}
                }).collect::<Vec<_>>();
                quote! {
                    (&#name::#var_name{ #(#left),* }, &#name::#var_name{ #(#right),* }) => {
                       #gen
                    }
                }
            },
            &syn::VariantData::Unit => {
                quote! {
                    (&#name::#var_name, &#name::#var_name) => {
                        return None;
                    }
                }
            },
        };
        differs.push(diff);
    }
    return quote! {
        impl ::struct_diff::Diff for #name {
            type Value = #name;

            #[allow(unreachable_patterns)]
            fn diff<'a>(&'a self, other: &'a #name) -> Option<Vec<::struct_diff::Difference<'a>>> {
                let mut diffs = Vec::with_capacity(1);
                match (self, other) {
                    #(#differs),*
                    _ => {
                        diffs.push(::struct_diff::Difference { field: "self".into(), left: self, right: other });
                    }
                }
                if diffs.len() > 0 {
                    return Some(diffs);
                }
                None
            }
        }
    }
}

/// Generates Diff impl for enum fields
struct TupleFieldsGenerator<'a> {
    fields: &'a [syn::Field],
}

impl<'a> quote::ToTokens for TupleFieldsGenerator<'a> {
    fn to_tokens(&self, tokens: &mut quote::Tokens) {
        for (field_name, _) in self.fields.iter().enumerate() {
                let field_name_s = field_name.to_string();
                use quote::Ident;
                let field_name = Ident::new(format!("{}", field_name));
                tokens.append(
                    quote!{
                        if let Some(inner_diffs) = self.#field_name.diff(&other.#field_name) {
                            for diff in inner_diffs {
                                let mut path = String::from(#field_name_s);
                                if !diff.field.is_empty() {
                                    path.push_str(&".");
                                }
                                path.push_str(&diff.field);
                                diffs.push(::struct_diff::Difference {
                                    field: path,
                                    left: diff.left,
                                    right: diff.right,
                                })
                            }
                        }
                    }
                );
        }
    }
}

/// Implements Diff for structs
fn impl_diff_struct(name: &syn::Ident, struct_: &syn::VariantData) -> quote::Tokens {
    match struct_ {
        &syn::VariantData::Struct(ref fields) => {
            let gen = StructGenerator { fields };
            return quote! {
                impl ::struct_diff::Diff for #name {
                    type Value = #name;

                    fn diff<'a>(&'a self, other: &'a #name) -> Option<Vec<::struct_diff::Difference<'a>>> {
                        let mut diffs = Vec::new();
                        #gen
                        if diffs.len() > 0 {
                            return Some(diffs);
                        }
                        return None;
                    }
                }
            }
        },
        &syn::VariantData::Tuple(ref fields) => {
            let gen = TupleFieldsGenerator { fields };
            return quote! {
                impl ::struct_diff::Diff for #name {
                    type Value = #name;

                    fn diff<'a>(&'a self, other: &'a #name) -> Option<Vec<::struct_diff::Difference<'a>>> {
                        let mut diffs = Vec::new();
                        #gen
                        if diffs.len() > 0 {
                            return Some(diffs);
                        }
                        return None;
                    }
                }
            }
        },
        v@_ => {
            /* only structs and tuples are supported for now */
            panic!("Support for {:?} not implemented", v);
        }
    }
}
