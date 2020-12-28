#![feature(proc_macro_diagnostic)]
#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use {
    syn::{parse_macro_input, parenthesized, braced, Path, LitStr, Ident, token},
    syn::parse::{Parse, ParseStream, Result},
    proc_macro::TokenStream,
    proc_macro2
};

#[proc_macro]
pub fn build_routes(input: TokenStream) -> TokenStream {
    let root: RouteTree = parse_macro_input!(input as RouteTree);
    let method_chain: proc_macro2::TokenStream = build_method_chain(root.scopes);

    let expanded = quote!{
        pub fn routes(config: &mut web::ServiceConfig) {
            config #method_chain;

        }

    };
    TokenStream::from(expanded)
}

struct RouteTree {
    scopes: Vec<Scope>
}

struct Scope {
    path: LitStr,
    scopes: Vec<Scope>,
    resources: Vec<Resource>
}

struct Resource {
    method: Ident,
    path: LitStr,
    action: Path
}

impl Parse for RouteTree {
    fn parse(stream: ParseStream) -> Result<Self> {
        let mut scopes: Vec<Scope> = Vec::new();

        let mut within_scope;
        let mut path_stream;
        while !stream.is_empty(){
            let _scope: Ident = stream.parse()?;
            parenthesized!( path_stream in stream );
            braced!(within_scope in stream);
            let path: LitStr = path_stream.parse()?;
            if let Ok(scp) = parse_scope(path, &within_scope) {
                scopes.push( scp );
            } else {
                within_scope.span().unstable().error("The the scope is malformatted here").emit()
            }
        }

        Ok(
            RouteTree {
                scopes
            }
        )
    }
}

fn parse_scope(path: LitStr, stream: ParseStream) -> Result<Scope> {
    let mut scopes: Vec<Scope> = Vec::new();
    let mut resources: Vec<Resource> = Vec::new();

    while !stream.is_empty() {
        let is_scope = stream.peek2(token::Paren);
        let path_stream;
        let within_scope;
        let path: LitStr;

        if is_scope {
            let _scope: Ident = stream.parse()?;
            parenthesized!( path_stream in stream );
            braced!(within_scope in stream);
            path = path_stream.parse()?;

            if let Ok(scp) = parse_scope(path, &within_scope) {
                scopes.push( scp);
            } else {
                within_scope.span().unstable().error("The the scope is malformatted here").emit()
            }

        } else {
            let method: Ident = stream.parse()?;
            path = stream.parse()?;
            let _at: Ident = stream.parse()?;
            let action: Path = stream.parse()?;

            resources.push(
                Resource {
                    method,
                    path,
                    action
                }
            );
        }
    }

    Ok(
        Scope {
            path,
            scopes,
            resources
        }
    )
}

fn build_method_chain(scopes: Vec<Scope>) -> proc_macro2::TokenStream {
    let mut method_chain: Vec<proc_macro2::TokenStream> = Vec::new();

    for scope in scopes {
        method_chain.push( build_scope_chain(&scope) );
    }

    quote! { #( #method_chain )*  }

}

fn build_scope_chain(scope: &Scope) -> proc_macro2::TokenStream {
    let mut method_chain: Vec<proc_macro2::TokenStream> = Vec::new();
    let path = &scope.path;

    for scope in &scope.scopes {
        method_chain.push( build_scope_chain( scope ) )
    }

    for resource in &scope.resources {
        method_chain.push( build_resource_call( resource ) )
    }

    quote!{ .service( actix_web::web::scope( #path ) #( #method_chain )*) }
}

fn build_resource_call(resource: &Resource) -> proc_macro2::TokenStream {
    let path = &resource.path;
    let method = &resource.method;
    let action = &resource.action;

    quote!{ .route( #path, actix_web::web::#method().to( #action ) ) }
}