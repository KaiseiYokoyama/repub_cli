pub use message::error::RepubError;
pub use message::warning::RepubWarning;

pub mod message {
    pub mod error {
        use std::fmt::{Display, Formatter};
        use colored::Colorize;

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
                use colored::*;
                write!(f, "{} {}","[Error]".red().bold(), self.as_ref())
            }
        }
    }

    pub mod warning {
        use std::fmt::{Display, Formatter};
        use colored::Colorize;

        pub struct RepubWarning(pub String);

        impl AsRef<str> for RepubWarning {
            fn as_ref(&self) -> &str {
                let RepubWarning(s) = &self;
                s.as_str()
            }
        }

        impl From<&str> for RepubWarning {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl Display for RepubWarning {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                use colored::*;
                write!(f, "{} {}","[Warning]".yellow().bold(), self.as_ref())
            }
        }
    }
}