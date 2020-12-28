#![feature(proc_macro_diagnostic)]
#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2;
use syn::spanned::Spanned;

use syn::{ Fields, Item, ItemStruct };

#[proc_macro_derive(Server)]
pub fn derive_server(item: TokenStream) -> TokenStream {
    let parsed_obj: Item = syn::parse(item).expect("failed to parse server configuration struct");

    match parsed_obj {
        Item::Struct(ref obj) => {
            let server: proc_macro2::TokenStream = impl_server(&obj);

            return TokenStream::from(server)
        },
        _ => { parsed_obj.span().unstable().error("This is not a struct").emit()
        }
    };

    TokenStream::new()
}

fn impl_server(obj: &ItemStruct) -> proc_macro2::TokenStream {
    let obj_name = &obj.ident;

    let mut has_cors_hosts: bool = false;
    match obj.fields {
        Fields::Named(ref fields) => {
            for field in &fields.named {
                if let Some(_i @ "cors_hosts") = field.ident.as_ref() {
                    has_cors_hosts = true;
                }
            }

        },
        _ => ()
    };

    let cors_hosts: proc_macro2::TokenStream = cors_hosts_method(has_cors_hosts);

    quote!{
        impl #obj_name {
            #cors_hosts

            pub fn run(self) -> std::io::Result<()> {
                actix_web::HttpServer::new(|| actix_web::App::new().
                    wrap( self.logger() ).
                    wrap( *self.cors() ).
                    configure( routes::routes )
                ).
                bind( std::env::var("HOST").expect("HOST must be set in environment variables") )?.
                run()
            }

            fn logger(self) -> actix_web::middleware::Logger {
                actix_web::middleware::Logger::default()
            }

            fn cors(self) -> Box<actix_cors::Cors> {
                let mut cors = Box::<actix_cors::Cors>::new();
                for cors_host in self.cors_hosts() {
                    cors.as_ref().allowed_origin( cors_host.as_ref() );
                }
                cors
            }
        }
    }
}

fn cors_hosts_method(exists: bool) -> proc_macro2::TokenStream {

    if exists {
        quote! {
            fn cors_hosts(self) -> Vec<String> {
                self.cors_hosts
            }
        }
    } else {
        quote! {
            fn cors_hosts(self) -> Vec<String> {
                Vec<String>::new()
            }
        }
    }

}