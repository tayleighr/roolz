#![feature(proc_macro_diagnostic)]
#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use {
    syn::{ Ident, Fields, Item, Type, ItemEnum, Visibility},
    proc_macro::TokenStream,
    proc_macro2,
    syn::spanned::Spanned
};

struct NewError<'a> {
    vis: &'a Visibility,
    name: &'a Ident,
    variants: Vec<Variant<'a>>
}
#[derive(Clone)]
struct Variant<'a> {
    kind: &'a Ident,
    source: &'a Type,
    code: &'a Type
}

#[proc_macro_attribute]
pub fn register_errors(_attr: TokenStream, input: TokenStream) -> TokenStream {

    let parsed_error_enum: Item = syn::parse(input).expect("failed to parse error enum");

    match parsed_error_enum {
        Item::Enum(ref error_enum) => {

            let variants = parse_variants(&error_enum, &parsed_error_enum);
            let new_error = NewError {name: &error_enum.ident, variants: variants.to_vec(), vis: &error_enum.vis};
            let app_error_impl = impl_app_error_from(&new_error);
            let std_error_impl = impl_std_error(&new_error);
            let from_impls = impl_from_variants(&new_error);
            let reformatted = reformatted_error_enum(&new_error);
            let display = impl_new_error_display(&new_error);

            let expanded = quote!{
                #reformatted
                #display
                #app_error_impl
                #std_error_impl
                #from_impls
            };

            let tk = TokenStream::from(expanded);
//            print!{"{:?}\n", &tk.to_string()}
            return tk

        },
        _ => parsed_error_enum.span().unstable().error("This is not an enum").emit()
    };

    TokenStream::new()
}

fn reformatted_error_enum(new_error: &NewError) -> proc_macro2::TokenStream {

    let vis = new_error.vis;
    let name = new_error.name;
    let mut variants: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let kind = variant.kind;
        let source = variant.source;
        variants.push(
            quote!{
                #kind(#source),
            }
        )
    }
    quote!{
        #[derive(Debug)]
        #vis enum #name {
            #( #variants )*
        }
    }
}

fn impl_new_error_display(new_error: &NewError ) -> proc_macro2::TokenStream {

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    let kind = &new_error.name;

    for variant in &new_error.variants {
        let variant_kind = variant.kind;
        match_statements.push(
            quote!{ #kind::#variant_kind( ref e  ) => e.fmt(f), }
        )
    }

    quote!{
        impl std::fmt::Display for #kind {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self {
                    #( #match_statements )*
                }
            }
        }
    }
}

fn impl_app_error_from(new_error: &NewError) -> proc_macro2::TokenStream {
    let kind = &new_error.name;

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let variant_kind = variant.kind;
        let variant_code = variant.code;

        match_statements.push(
            quote!{ #kind::#variant_kind(_) => #variant_code, }
        )
    }

    let kind_string = format!("{:?}", kind);

    quote!{
        impl From<#kind> for AppError {
            fn from(error: #kind) -> Self {
                let source_kind = match error.source() {
                    Some(e) => Some(e.to_string()),
                    _=> None
                };

                let code = match error {
                    #( #match_statements )*
                };

                AppError {
                    code,
                    source_kind,
                    kind: Some(#kind_string),
                    message: error.to_string(),
                }
            }
        }

    }
}

//fn impl_new_error_from_app_error(new_error: &NewError) -> proc_macro2::TokenStream {
//    let kind = &new_error.name;
//
//    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();
//
//    for variant in &new_error.variants {
//        let variant_kind = variant.kind;
//        let variant_code = variant.code;
//
//        match_statements.push(
//            quote!{ #kind::#variant_kind(_) => #variant_code, }
//        )
//    }
//
//    quote!{
//        impl From<AppError> for #kind {
//            fn from(error: AppError) -> Self {
//                let source_kind = match error.source() {
//                    Some(e) => Some(e.to_string()),
//                    _=> None
//                };
//
//                let code = match error {
//                    #( #match_statements )*
//                };
//
//                AppError {
//                    code,
//                    source_kind,
//                    kind: Some(String::from("#kind")),
//                    message: error.to_string(),
//                }
//            }
//        }
//
//    }
//}

fn impl_std_error(new_error: &NewError) -> proc_macro2::TokenStream {

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    let kind = &new_error.name;

    for variant in &new_error.variants {
        let variant_kind = variant.kind;
        match_statements.push(
            quote!{ #kind::#variant_kind( ref e  ) => Some(e), }
        )
    }

    quote!{
        use std::error::Error;

        impl std::error::Error for #kind {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match *self {
                    #( #match_statements )*
                }
            }
        }
    }
}

fn impl_from_variants(new_error: &NewError) -> proc_macro2::TokenStream {
    let name = &new_error.name;
    let variants = &new_error.variants;

    let mut tokenized_impls: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in variants {
        let kind = variant.kind;
        let code = variant.code;
        let source = variant.source;
        tokenized_impls.push(
            quote!{
                impl From<#source> for #name {
                    fn from(e: #source) -> #name {
                            #name::#kind(e)

//                        AppError{
//                            source_kind: Some(String::from("#source")),
//                            code: #code,
//                            kind: Some(String::from("#kind")),
//                            message: error.to_string(),
//                        }
                    }
                }

            }
        )
    }

    quote!{
        #( #tokenized_impls )*
    }
}

fn parse_variants<'a>(error_enum: &'a ItemEnum, parsed_context: &'a Item) -> Vec<Variant<'a>>{
    let mut variants: Vec<Variant<'a>> = Vec::new();

    for variant in &error_enum.variants {
        let kind = &variant.ident;
        if let Some((source, code)) = parse_fields(variant, parsed_context){
            variants.push(
                Variant{ kind, source, code }
            )
        }
    }

    variants
}

fn parse_fields<'a>(variant: &'a syn::Variant, parsed_context: &'a Item) -> Option<(&'a Type, &'a Type)> {
    let fields = &variant.fields;

    match fields {
        Fields::Unnamed(ref named_fields) => {
            let mut pairs = named_fields.unnamed.pairs();
            if pairs.len() == 2 {
                let source_field = &pairs.next().unwrap().value().ty;
                let code_field = &pairs.next().unwrap().value().ty;
                return Some((source_field, code_field));
            } else {
                parsed_context.span().unstable().error("Each variant must define an error type and a StatusCode").emit()
            }
        },
        _=> parsed_context.span().unstable().error("Enum fields must be unnamed").emit()
    }

    parsed_context.span().unstable().error("Failed to parse variant fields").emit();

    None
}