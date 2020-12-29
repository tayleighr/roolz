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
    result_name: &'a Ident,
    variants: Vec<Variant<'a>>
}
#[derive(Clone)]
struct Variant<'a> {
    name: &'a Ident,
    source: &'a Type,
    code: Option<&'a Type>
}

#[proc_macro_attribute]
pub fn register_errors(_attr: TokenStream, input: TokenStream) -> TokenStream {

    let parsed_error_enum: Item = syn::parse(input).expect("failed to parse error enum");

    match parsed_error_enum {
        Item::Enum(ref error_enum) => {
            let name_space = &error_enum.ident;
            let variants = parse_variants(&error_enum, &parsed_error_enum);
            let error_name = proc_macro2::Ident::new(&format!("{:}{:}", name_space, "Error"), proc_macro2::Span::call_site());
            let result_name = proc_macro2::Ident::new(&format!("{:}{:}", name_space, "Result"), proc_macro2::Span::call_site());
            let new_error = NewError {name: &error_name, result_name: &result_name, variants: variants.to_vec(), vis: &error_enum.vis};

            let app_error_impl = impl_app_error_from(&new_error);
            let std_error_impl = impl_std_error(&new_error);
            let from_impls = impl_from_variants(&new_error);
            let reformatted = format_error_and_result(&new_error);
            let display = impl_new_error_display(&new_error);
//            let app_error_status_impls = impl_app_error_status(&new_error);

            let expanded = quote!{
//                #( #app_error_status_impls )*
                #reformatted
                #display
                #app_error_impl
                #std_error_impl
                #from_impls
            };

            return TokenStream::from(expanded);

        },
        _ => parsed_error_enum.span().unstable().error("This is not an enum").emit()
    };

    parsed_error_enum.span().unstable().error("Failed to parse error enum").emit();
    TokenStream::new()
}

fn format_error_and_result(new_error: &NewError) -> proc_macro2::TokenStream {

    let vis = new_error.vis;
    let name = new_error.name;
    let result = new_error.result_name;
    let mut variants: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let kind = variant.name;
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

        pub type #result<T> = std::result::Result<T, #name>;
    }
}

fn impl_new_error_display(new_error: &NewError ) -> proc_macro2::TokenStream {

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    let name = &new_error.name;

    for variant in &new_error.variants {
        let kind = variant.name;
        let kind_string = &kind.to_string();
        match_statements.push(
            quote!{
                #name::#kind( ref e ) => write!(f, "{:}", #kind_string),
            }
        )
    }

    quote!{
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self {
                    #( #match_statements )*
                }
            }
        }
    }
}

//fn impl_app_error_status(new_error: &NewError)  -> Vec<proc_macro2::TokenStream> {
//    let name = &new_error.name;
//
//    let mut a_e_s_impls: Vec<proc_macro2::TokenStream> = Vec::new();
//    for variant in &new_error.variants {
//        let body = if let Some(code) = &variant.code {
//            quote!{#code}
//        } else {
//            quote!{
//                let converter = #kind::from(error);
//                converter.code();
//            }
//        };
//
//        let kind = variant.name;
//
//        a_e_s_impls.push(
//            quote! {
//                impl AppErrorStatus<#kind> for #name {
//                    fn code<U>(&self, error: U) -> actix_web::http::StatusCode
//                    where #kind: AppErrorStatus<U> + std::convert::From<U> {
//                        #body
//                    }
//                }
//            }
//        )
//    }
//
//    a_e_s_impls
//}

fn impl_app_error_from(new_error: &NewError) -> proc_macro2::TokenStream {
    let name = &new_error.name;

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let kind = variant.name;
        let variant_code = variant.code;

        if let Some(code) = variant_code {
            match_statements.push(quote!{ #name::#kind(_) => #code, })
        } //else {
//            match_statements.push(
//                quote!{
//                    #name::#kind(ref e) => {
//                        <#name as AppErrorStatus<#kind>::code(e);
//                    },
//                }
//            )
//        }
    }

    let name_string = name.to_string();

    quote!{
        impl std::convert::From<#name> for AppError {
            fn from(error: #name) -> Self {
                let message = match error.source() {
                    Some(e) => e.to_string(),
                    _=> String::from("")
                };

                let code = match error {
                    #( #match_statements )*
                };

                AppError {
                    code,
                    message,
                    kind: Some(#name_string.to_string()),
                    source: Some(error.to_string()),
                }
            }
        }

    }
}

fn impl_std_error(new_error: &NewError) -> proc_macro2::TokenStream {

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    let name = &new_error.name;

    for variant in &new_error.variants {
        let kind = variant.name;
        match_statements.push(
            quote!{ #name::#kind( ref e  ) => Some(e), }
        )
    }

    quote!{
        use std::error::Error;

        impl std::error::Error for #name {
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
        let kind = variant.name;
        let source = variant.source;
        tokenized_impls.push(
            quote!{
                impl std::convert::From<#source> for #name {
                    fn from(e: #source) -> #name {
                        #name::#kind(e)

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
        let name = &variant.ident;
        if let Some((source, code)) = parse_fields(variant, parsed_context){
            variants.push(
                Variant{ name, source, code }
            )
        }
    }

    variants
}

fn parse_fields<'a>(variant: &'a syn::Variant, parsed_context: &'a Item) -> Option<(&'a Type, Option<&'a Type>)> {
    let fields = &variant.fields;

    match fields {
        Fields::Unnamed(ref named_fields) => {
            let mut pairs = named_fields.unnamed.pairs();
            if pairs.len() > 0 {
                let source_field = &pairs.next().unwrap().value().ty;
                let code_field = if let Some(code_pair) = &pairs.next() {
                    Some(&code_pair.value().ty)
                } else {
                    None
                };
                return Some((source_field, code_field));
            } else {
                parsed_context.span().unstable().error("Each variant must define an error type").emit()
            }
        },
        _=> parsed_context.span().unstable().error("Enum fields must be unnamed").emit()
    }

    parsed_context.span().unstable().error("Failed to parse variant fields").emit();

    None
}