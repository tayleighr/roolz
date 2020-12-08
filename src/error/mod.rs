pub use actix_web::{ http::StatusCode };
pub use std::fmt;
pub use super::roolz_error;
pub use super::include_error_types;


// --- definition --- //


pub trait RoolzError {
    fn status_code(&self) -> Option<StatusCode>;
}

// pub struct ApplicationError<T>(pub T); COULD BE GOOD WAY TO IMPLEMENT COMMON ERROR BEHAVIOR FROM DIFFERENT TYPES
#[macro_export]
macro_rules! include_error_types {
    {
        $(
            $( #[$meta:meta] )*
            $ErrorName:ident
        ),+ $(,)?
    } => {
        $(
            $(#[$meta])*
            #[derive(Debug)]
            pub struct $ErrorName {
                pub message: &'static str,
                pub status_code: Option<StatusCode>
            }

            impl fmt::Display for $ErrorName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{}", self.message)
                    }
            }

            impl std::error::Error for $ErrorName {
                fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                    None
                }
            }

            impl RoolzError for $ErrorName {
                fn status_code(&self) -> Option<StatusCode> {
                    self.status_code
                }
            }
        )?
    }
}

#[macro_export]
macro_rules! roolz_error {
    {
        $( #[$meta:meta] )*
        $pub:vis enum $ErrorName:ident {
            $(
                $Variant:ident( $TYPE:ty, $STATUS_ERROR:expr )
            ),+ $(,)?
        }

    } => {

        $(#[$meta])*
        #[derive(Debug)]
        $pub enum $ErrorName {
            $(
                $Variant($TYPE),
            )*
        }

        $(
            impl From<$TYPE> for $ErrorName {
                fn from(e: $TYPE) -> $ErrorName {
                    $ErrorName::$Variant(e)
                }
            }
        )?

        impl fmt::Display for $ErrorName {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    $(
                        $ErrorName::$Variant( ref e  ) => e.fmt(f),
                    )?
                }
            }
        }

        impl std::error::Error for $ErrorName {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match *self {
                    $(
                        $ErrorName::$Variant( ref e  ) => Some(e), //come back to here to make this some for not
                    )?
                }
            }
        }

        impl crate::error::RoolzError for $ErrorName {
            fn status_code(&self) -> Option<StatusCode> {
                match *self {
                    $(
                        $ErrorName::$Variant( ref _e  ) => Some($STATUS_ERROR),
                    )?
                }
            }
        }

    }
}