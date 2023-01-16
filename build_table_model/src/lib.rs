#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2;
use syn::spanned::Spanned;

use syn::{
     Path, Ident, Type, Attribute, Result,
    GenericArgument, PathArguments, Fields, Item, ItemStruct
};

use proc_macro_error::{proc_macro_error, emit_error};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn table_model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let model: Item = syn::parse(item).expect("failed to parse model struct");

    match model {
        Item::Struct(ref struct_model) => {
            let table_model: proc_macro2::TokenStream = build_model(&struct_model, &model);
            let proxy: proc_macro2::TokenStream = build_proxy(&struct_model, &model);
            let deps: proc_macro2::TokenStream = import_schema_dependencies(&struct_model.ident);
            let boxed_query: proc_macro2::TokenStream = build_boxed_query(&struct_model.ident);
            let crud: proc_macro2::TokenStream = build_crud_methods(&struct_model.ident);

            let expanded = quote! {
                #deps
                #table_model
                #proxy
                #boxed_query
                #crud
            };

            return TokenStream::from(expanded)

        },
            _ => emit_error!(proc_macro2::Span::call_site(), "This is not a struct")
    };

    TokenStream::new()
}

fn build_model(model: &ItemStruct, parsed_model: &Item) -> proc_macro2::TokenStream {
    let table = &model.ident.to_string();
    let attributes: &Vec<Attribute> = &model.attrs;

    let mut tokenized_fields: Vec<proc_macro2::TokenStream> = Vec::new();
    match model.fields {
        Fields::Named(ref fields) => {
            for field in &fields.named {
                tokenized_fields.push(quote!{ #field, } )
            }

        },
        _ => emit_error!(proc_macro2::Span::call_site(), "The struct must contain only named fields")
    };

    quote!{
        #[derive(Identifiable, Eq, PartialEq, Debug, Queryable, QueryableByName, Serialize, AsChangeset)]
        #( #attributes )*
        #[table_name=#table]
        pub struct Model {
            #( #tokenized_fields )*
        }
    }
}

fn build_proxy(model: &ItemStruct, parsed_model: &Item) -> proc_macro2::TokenStream {
    let table = &model.ident.to_string();
    let attributes: &Vec<Attribute> = &model.attrs;

    let mut tokenized_fields: Vec<proc_macro2::TokenStream> = Vec::new();

    match model.fields {
        Fields::Named(ref fields) => {
            for field in &fields.named {
                if let Ok(typ) = extract_type_from_option(&field.ty, &parsed_model) {
                    let name = &field.ident;
                    tokenized_fields.push( quote! { #name : Option<#typ>, } );
                } else {
                    emit_error!(proc_macro2::Span::call_site(), "Failed to parse struct types");
                }

            }
        },
        _ => emit_error!(proc_macro2::Span::call_site(), "The struct must contain named fields")
    };

    quote!{
        #[derive(Identifiable, Insertable, Deserialize, AsChangeset, Debug, Clone)]
        #( #attributes )*
        #[table_name=#table]
        pub struct Proxy {
            #( #tokenized_fields )*
        }
    }
}

fn import_schema_dependencies(table: &Ident) -> proc_macro2::TokenStream {
    quote!{
        use crate::schema::#table;
        use crate::schema::#table::dsl::*;
    }
}

fn build_boxed_query(table: &Ident) -> proc_macro2::TokenStream {
    quote!{
        fn boxed_query() -> #table::BoxedQuery<'static, roolz::db::db_type> {
            #table::table.into_boxed()
        }
    }
}

fn build_crud_methods(table: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl Proxy {
            pub fn create(&mut self) -> AppResult<Model> {
                self.id = None;
                match diesel::insert_into(#table).values(&*self).get_result(&db()) {
                    Ok(record) => Ok(record),
                    Err(e) => Err( DBError::for_app(e) )
                }
            }

            pub fn update(&mut self) -> AppResult<Model> {
                if let Some(i_d) = self.id {
                    if record_exists(i_d) {
                        match diesel::update(#table.find(i_d)).set(&*self).get_result(&db()) {
                            Ok(record) => Ok(record),
                            Err(e) => Err( DBError::for_app(e) )
                        }
                    } else {
                        Err( not_found("No record exists for this ID") )
                    }
                } else {
                    Err( not_found("No ID provided for update") )
                }
            }
        }

        pub fn find(pkey: i32) -> AppResult<Model> {
            match #table.find(pkey).get_result(&db()) {
                Ok(model) => Ok(model),
                Err(_e) => Err( not_found("No record exists for this ID") )
            }
        }

        pub fn delete(pkey: i32) -> AppResult<Model> {
            if record_exists(pkey) {
                match diesel::delete(#table.find(pkey)).get_result(&db()) {
                    Ok(model) => Ok(model),
                    Err(_e) => Err( not_found("No record exists for this ID") )
                }
            } else {
                Err( not_found("No record exists for this ID") )
            }

        }

        pub fn record_exists(pkey: i32) -> bool {
            let query = #table.find(pkey);

            match select(exists(query)).get_result(&db()) {
                Ok(result) => result, Err(_e) => false
            }
        }
    }
}


// THIS HEINOUS SHIT MAKES ME WANNA HAVE THE TROTS

fn extract_type_from_option<'a>(ty: &'a Type, parsed_model: &'a Item) -> Result<&'a Type> {
    fn path_is_option(path: &Path) -> bool {
        path.segments.len() == 1
            && path.segments.iter().next().unwrap().ident == "Option"
    }

    match ty {
        Type::Path(typepath) => {
            if typepath.qself.is_none() && path_is_option(&typepath.path) {

                // Get the first segment of the path (there is only one, in fact: "Option"):
                let type_params = &typepath.path.segments[0].arguments;

                // It should have only on angle-bracketed param ("<String>"):
                match type_params {
                    PathArguments::AngleBracketed(params) => {
                        let type_arg = &params.args[0];

                        // This argument must be a type:
                        match type_arg {
                            GenericArgument::Type(ty) => {
                                return Ok(&ty)
                            },
                            _ => emit_error!(proc_macro2::Span::call_site(), "Expecting an inner type")
                        }
                    },
                    _ => return Ok(&ty)
                };
            }
        },
        _ => emit_error!(proc_macro2::Span::call_site(), "Expecting A type definition")
    }

    Ok(&ty)
}