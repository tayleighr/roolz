#![feature(proc_macro_diagnostic)]
#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use {
    syn::{ parse_macro_input, Ident, Fields, Item, Type, ItemEnum, Visibility, AttributeArgs, NestedMeta, Lit, LitInt },
    proc_macro::TokenStream,
    proc_macro2,
    syn::spanned::Spanned
};

type Codes = Vec<LitInt>;

#[derive(Clone)]
struct NewError {
    vis: Visibility,
    name: Ident,
    result_name: Ident,
    results_name: Ident,
    variants: Vec<Variant>
}

#[derive(Clone)]
struct Variant {
    name: Ident,
    source: Type,
    code: Option<LitInt>
}

#[proc_macro_attribute]
pub fn register_errors(attrs: TokenStream, input: TokenStream) -> TokenStream {

    let parsed_attrs: AttributeArgs = parse_macro_input!(attrs as AttributeArgs);
    let parsed_error_enum: Item = syn::parse(input).expect("failed to parse error enum");

    let mut expand_segments: Vec<proc_macro2::TokenStream> = Vec::new();

    match parsed_error_enum {
        Item::Enum(ref enum_item) => {

            let new_error = new_error(&enum_item, &parsed_attrs, &parsed_error_enum);
            expand_segments.push(new_types(&new_error));
            expand_segments.push(impl_error_status(&new_error));
            expand_segments.push(impl_std_error(&new_error));
            expand_segments.push(impl_display(&new_error));
            expand_segments.push(impl_app_error_conversion(&new_error));
            expand_segments.push(impl_variant_conversion(&new_error));
        },
        _ => parsed_error_enum.span().unstable().error("This is not an enum").emit()
    };

    let expanded = quote!{
        #( #expand_segments )*
    };

    TokenStream::from(expanded)
}

fn new_error(enum_item: &ItemEnum, parsed_attrs: &AttributeArgs, parsed_enum: &Item) -> NewError {
    let name_space = &enum_item.ident;
    let variants = parse_variants(&enum_item, parsed_attrs, &parsed_enum);

    let error_name = proc_macro2::Ident::new(&format!("{:}{:}", name_space, "Error"), proc_macro2::Span::call_site());
    let result_name = proc_macro2::Ident::new(&format!("{:}{:}", name_space, "Result"), proc_macro2::Span::call_site());
    let results_name = proc_macro2::Ident::new(&format!("{:}{:}", name_space, "Results"), proc_macro2::Span::call_site());

    NewError {name: error_name, result_name: result_name, results_name: results_name, variants: variants, vis: enum_item.vis.clone()}
}

fn new_types(new_error: &NewError) -> proc_macro2::TokenStream {

    let vis = &new_error.vis;
    let name = &new_error.name;
    let result = &new_error.result_name;
    let results = &new_error.results_name;

    let mut variants: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let kind = &variant.name;
        let source = &variant.source;

        variants.push( quote!{ #kind(#source), } );
    }

    quote!{

        #[derive(Debug)]
        #vis enum #name {
            #( #variants )*
            App(AppError)
        }

        pub type #result<T> = std::result::Result<T, #name>;
        pub type #results<T> = std::result::Result<T, #name>;
    }
}

fn impl_error_status(new_error: &NewError) -> proc_macro2::TokenStream {
    let name = &new_error.name;
    let name_string = name.to_string();

    let mut code_match_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut kind_match_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut origin_match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let kind = &variant.name;
        let kind_string = kind.to_string();
        let source = &variant.source;
        let source_string = quote!{#source}.to_string();

        if let Some(code) = &variant.code {
            code_match_statements.push(
                quote!{
                    #name::#kind(_) => {
                        if let Ok(c) = actix_web::http::StatusCode::from_u16(#code) {
                            Some(c)
                        } else {
                            Some(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    },
                }
            )
        } else {
            code_match_statements.push(
                quote!{ #name::#kind(_) => None, }
            )
        }

        kind_match_statements.push(
            quote!{ #name::#kind(_) => String::from(#kind_string), }
        );

        origin_match_statements.push(
            quote!{ #name::#kind(_) => String::from(#source_string), }
        );
    }

    quote!{
        impl roolz::error::ErrorMeta for #name {
            fn code(&self) -> Option<actix_web::http::StatusCode> {
                match &*self {
                    #( #code_match_statements )*
                    #name::App(e) => e.code()
                }
            }

            fn typ(&self) -> String {
               String::from(#name_string)
            }

            fn kind(&self) -> String {
                match &*self {
                    #( #kind_match_statements )*
                    #name::App(ref e) => e.kind()
                }
            }

            fn origin(&self) -> String {
                match &*self {
                    #( #origin_match_statements )*
                    #name::App(ref e) => e.origin()
                }
            }

            fn reason(&self) -> String {
                use std::error::Error;

                if let Some(source) = self.source() {
                    source.to_string()
                } else {
                    String::from("No reason provided")
                }
            }

        }
    }
}


fn impl_std_error(new_error: &NewError) -> proc_macro2::TokenStream {
    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    let name = &new_error.name;

    for variant in &new_error.variants {
        let kind = &variant.name;
        match_statements.push(
            quote!{ #name::#kind( ref e  ) => Some(e), }
        )
    }

    quote!{
        use std::error::Error;

        impl std::error::Error for #name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match &*self {
                    #( #match_statements )*
                    #name::App(ref e) => Some(e)
                }
            }
        }
    }
}

fn impl_display(new_error: &NewError ) -> proc_macro2::TokenStream {

    let mut match_statements: Vec<proc_macro2::TokenStream> = Vec::new();

    let name = &new_error.name;

    for variant in &new_error.variants {
        let kind = &variant.name;
        match_statements.push(
            quote!{
                #name::#kind( ref e ) => e.fmt(f),
            }
        )
    }

    quote!{
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self {
                    #( #match_statements )*
                    #name::App( ref e ) => e.fmt(f)
                }
            }
        }
    }
}

fn impl_app_error_conversion(new_error: &NewError) -> proc_macro2::TokenStream {
    let name = &new_error.name;

    quote!{
        impl std::convert::From<#name> for AppError {
            fn from(error: #name) -> Self {
                AppError::From(Box::new(error))
            }
        }

        impl std::convert::From<AppError> for #name {
            fn from(error: AppError) -> Self {
                #name::App(error)
            }
        }
    }
}

fn impl_variant_conversion(new_error: &NewError) -> proc_macro2::TokenStream {
    let name = &new_error.name;

    let mut tokenized_impls: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &new_error.variants {
        let kind = &variant.name;
        let source = &variant.source;

        tokenized_impls.push(
            quote!{
                impl std::convert::From<#source> for #name {
                    fn from(e: #source) -> #name {
                        #name::#kind(e)
                    }
                }

                impl roolz::error::ForApp<#source> for #name {
                    fn for_app(e: #source) -> AppError {
                        #name::from(e).into()
                    }
                }
            }
        );
    }

    quote!{
        #( #tokenized_impls )*
    }
}

fn parse_variants(error_enum: &ItemEnum, parsed_attrs: &AttributeArgs, parsed_context: &Item) -> Vec<Variant>{
    let mut variants: Vec<Variant> = Vec::new();

    let codes = code_list(&parsed_attrs);
    let mut code_iter = codes.iter();

    for variant in &error_enum.variants {
        if let Some(source) = parse_fields(variant, parsed_context){
            let code = if let Some(code) = code_iter.next() {
                Some(code.to_owned())
            } else {
                None
            };

            variants.push( Variant { name: variant.ident.clone(), source: source, code } )
        }
    }

    variants
}

fn code_list(parsed_attrs: &AttributeArgs) -> Codes {
    let mut code_list: Vec<LitInt> = Vec::new();

    for nested_code_attr in parsed_attrs.into_iter() {
        match nested_code_attr {
            NestedMeta::Lit(code_lit_enum) => {
                match code_lit_enum {
                    Lit::Int(code) => {
                        code_list.push(code.to_owned())
                    },_=>{}
                }
            }, _=> {}
        }
    }

    code_list
}

fn parse_fields(variant: &syn::Variant, parsed_context: &Item) -> Option<Type> {
    match &variant.fields {
        Fields::Unnamed(ref named_fields) => {
            if let Some(source_field) = &named_fields.unnamed.pairs().next() {
                return Some(source_field.value().ty.clone());
            } else {
                parsed_context.span().unstable().error("Each variant must define an error type").emit()
            }
        },
        _=> parsed_context.span().unstable().error("Enum fields must be unnamed").emit()
    }

    parsed_context.span().unstable().error("Failed to parse variant fields").emit();

    None
}