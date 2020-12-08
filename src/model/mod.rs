// private imports
use super::error::*;

// common libraries
pub use serde::{ Serialize, Deserialize };
pub use super::diesel::{
    RunQueryDsl,
    prelude::*,
    dsl::*
};

pub use super::db::db;


// --- definition --- //


// result type
pub type RecordResult<T> = std::result::Result<T, self::Error>;

// defining and creating error types for model

include_error_types! {
    NotFound,
    Conflict
}

roolz_error! {
    pub enum Error {
        DBConflict(diesel::result::Error, StatusCode::CONFLICT),
        DBConnection(diesel::result::ConnectionError, StatusCode::SERVICE_UNAVAILABLE),
        NotFound(NotFound, StatusCode::NOT_FOUND),
        Conflict(Conflict, StatusCode::CONFLICT)
    }
}

pub fn not_found(message: &'static str) -> self::NotFound {
    let e: self::NotFound = self::NotFound{ message: message, status_code: Some(StatusCode::NOT_FOUND) };
    e
}

pub fn conflict(message: &'static str) -> self::Conflict {
    let e: self::Conflict = self::Conflict{message: message, status_code: Some(StatusCode::CONFLICT)};
    e
}

// composes the common objects, proxies, and methods for the model
#[macro_export]
macro_rules! model {
    (
        $( #[$meta_variables:meta] )*
        $table:ident ( $table_string:literal ): $input:tt
    ) => {
        table_model! { $( #[ $meta_variables ] )* $table_string $input }
        model_proxy! { $table_string $input }
        boxed_query_method! { $table }
        table_imports! { $table }
        crud_methods! { $table }
    };
}

#[macro_export]
macro_rules! table_imports {
    ($table:ident) => {
        use crate::schema::$table;
        use crate::schema::$table::dsl::*;
    }
}

// composes the boxed query method for the model
#[macro_export]
macro_rules! boxed_query_method {
    ($table:ident) => {
        fn boxed_query() -> $table::BoxedQuery<'static, roolz::db::db_type> {
            $table::table.into_boxed()
        }
    }
}

// composes crud methods for model
#[macro_export]
macro_rules! crud_methods {
    ( $table:ident ) => {
        pub fn create(proxy: &mut Proxy) -> RecordResult<Model> {

            match diesel::insert_into($table).values(&*proxy).get_result(&db()) {
                Ok(record) => Ok(record),
                Err(e) => Err(Error::from(e))
            }
        }

        pub fn update(pkey: i32, proxy: &mut Proxy) -> RecordResult<Model> {
            if record_exists(pkey) {
                match diesel::update($table.find(pkey)).set(&*proxy).get_result(&db()) {
                    Ok(record) => Ok(record),
                    Err(e) => Err(Error::from(e))
                }
            } else {
                Err(Error::from( not_found("No record exists for this ID") ))
            }
        }

        pub fn find(pkey: i32) -> RecordResult<Model> {
            match $table.find(pkey).get_result(&db()) {
                Ok(model) => Ok(model),
                Err(_e) => Err(Error::from( not_found("No record exists for this ID") ))
            }
        }

        pub fn delete(pkey: i32) -> RecordResult<Model> {
            if record_exists(pkey) {
                match diesel::delete($table.find(pkey)).get_result(&db()) {
                    Ok(model) => Ok(model),
                    Err(_e) => Err(Error::from( not_found("No record exists for this ID") ))
                }
            } else {
                Err(Error::from( not_found("No record exists for this ID") ))
            }

        }

        pub fn record_exists(pkey: i32) -> bool {
            let query = $table.find(pkey);

            match select(exists(query)).get_result(&db()) {
                Ok(result) => result, Err(_e) => false
            }
        }
    }
}

// composes pub struct Model for representations of the data coming out of the db table
#[macro_export]
macro_rules! table_model {
// the final composition of the Model struct
    (
        @munch [] -> {
            $( #[ $meta_variables:meta ] )* $table:literal $( + $column_name:ident: $type:ty )*
        }
    ) => {
        #[derive(Eq, PartialEq, Debug, Queryable, Serialize, AsChangeset)]
        $( #[ $meta_variables ] )*
        #[table_name=$table]
        pub struct Model {
            $( $column_name: $type ), *
        }
    };

// the case where no proxies are provided, and this is the final column to process
    (
        @munch [ $column_name:ident: $type:ty ] -> { $( $output:tt )* }
    ) => {
        table_model!( @munch [] -> { $( $output )* + $column_name: $type } );
    };

// the case where no proxies are provided, and there are more columns to process
    (
        @munch [ $column_name:ident: $type:ty, $( $more_attributes:tt )* ] -> { $( $output:tt )* }
    ) => {
        table_model!( @munch [ $( $more_attributes )* ] -> { $( $output )* + $column_name: $type } );
    };

// the entry point for outside calls
    (
        $( #[ $meta_variables:meta ] )* $table:literal { $( $input:tt )* }
    ) => {
        table_model!(
            @munch [ $( $input )* ] -> { $( #[ $meta_variables ] )* $table }
        );
    };
}

// composes the model's Proxy struct, which is a 1 to 1 representation of the model, but with
// optional fields, used for db queries and inserts
#[macro_export]
macro_rules! model_proxy {
// the final composition of the Model struct
    (
        @munch [] -> {
            $table:literal $( + $column_name:ident: $type:ty )*
        }
    ) => {
        #[derive(Insertable, Deserialize, AsChangeset, Debug)]
        #[table_name=$table]
        pub struct Proxy {
            $($column_name: Option< $type >), *
        }
    };

// the case where no proxies are provided, and this is the final column to process
    (
        @munch [ $column_name:ident: Option< $type:ty > ] -> { $( $output:tt )* }
    ) => {
        model_proxy!( @munch [] -> { $( $output )* + $column_name: $type } );
    };

// the case where no proxies are provided, and this is the final column to process
    (
        @munch [ $column_name:ident: $type:ty ] -> { $( $output:tt )* }
    ) => {
        model_proxy!( @munch [] -> { $( $output )* + $column_name: $type } );
    };

// the case where no proxies are provided, and there are more columns to process
    (
        @munch [ $column_name:ident: Option< $type:ty, $( $more_attributes:tt )* ] -> { $( $output:tt )* }
    ) => {
        model_proxy!( @munch [ $( $more_attributes )* ] -> { $( $output )* + $column_name: $type } );
    };

// the case where no proxies are provided, and there are more columns to process
    (
        @munch [ $column_name:ident: $type:ty, $( $more_attributes:tt )* ] -> { $( $output:tt )* }
    ) => {
        model_proxy!( @munch [ $( $more_attributes )* ] -> { $( $output )* + $column_name: $type } );
    };

// the entry point for outside calls
    (
        $table:literal { $( $input:tt )* }
    ) => {
        model_proxy!(
            @munch [ $( $input )* ] -> { $table }
        );
    };
}