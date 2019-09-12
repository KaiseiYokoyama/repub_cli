pub use message::error::RepubError;

pub mod message {
    pub mod error {
        use std::fmt::{Display, Formatter};

        pub struct RepubError(pub String);

        impl AsRef<str> for RepubError {
            fn as_ref(&self) -> &str {
                let RepubError(s) = &self;
                s.as_str()
            }
        }

        impl From<&str> for RepubError {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl Display for RepubError {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "[Error] {}", &self)
            }
        }
    }
}