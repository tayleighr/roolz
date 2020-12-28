#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use syn::{parse_macro_input, braced, Ident, Token, Path};
use syn::parse::{Parse, ParseStream, Result};
use proc_macro::TokenStream;
use proc_macro2;

#[proc_macro]
pub fn register_errors(input: TokenStream) -> TokenStream {
    let error_tree: ErrorTree = parse_macro_input!(input as ErrorTree);
    let from_impls: proc_macro2::TokenStream = impl_from(error_tree.errors);

    let expanded = quote!{
        #from_impls
    };
    TokenStream::from(expanded)
}

struct ErrorTree {
    errors: Vec<ErrorData>
}

struct ErrorData {
    kind: Path,
    code: Path
}

impl Parse for ErrorTree {
    fn parse(stream: ParseStream) -> Result<Self> {
        let mut errors: Vec<ErrorData> = Vec::new();

        let mut error_data;
        while !stream.is_empty(){
            braced!(error_data in stream);
            let _kind: Ident = error_data.parse()?;
            let _colon: Token![:] = error_data.parse()?;
            let kind: Path = error_data.parse()?;
            let _comma: Token![,] = error_data.parse()?;
            let _code: Ident = error_data.parse()?;
            let _colon: Token![:] = error_data.parse()?;
            let code: Path = error_data.parse()?;
            errors.push(
                ErrorData{
                    kind,
                    code
                }
            );
        }

        Ok(
            ErrorTree {
                errors
            }
        )
    }
}


fn impl_from(errors: Vec<ErrorData>) -> proc_macro2::TokenStream {
    let mut tokenized_impls: Vec<proc_macro2::TokenStream> = Vec::new();

    for error_data in errors {
        let kind = error_data.kind;
        let code = error_data.code;
        tokenized_impls.push(
            quote!{
                impl From<#kind> for AppError {
                    fn from(error: #kind) -> Self {
                        AppError {
                            code: #code,
                            kind: Some(String::from("#kind")),
                            message: error.to_string(),
                        }
                    }
                }

            }
        )
    }

    quote!{
        #( #tokenized_impls )*
    }
}