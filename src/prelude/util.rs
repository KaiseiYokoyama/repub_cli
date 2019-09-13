pub use message::{
    Message,
    error::RepubError,
    warning::RepubWarning,
};

use crate::prelude::*;

pub mod message {
    use super::*;

    /// ログレベルに関係なく表示されるメッセージ
    pub trait Message: Display {
        fn print(&self) {
            println!("{}", &self)
        }
    }

    pub mod error {
        use super::*;

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
                write!(f, "{} {}", "[Error]".red().bold(), self.as_ref())
            }
        }

        impl Message for RepubError {}
    }

    pub mod warning {
        use super::*;

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
                write!(f, "{} {}", "[Warning]".yellow().bold(), self.as_ref())
            }
        }

        impl Message for RepubWarning {}
    }
}