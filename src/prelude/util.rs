pub use message::{
    Message,
    error::RepubError,
    warning::RepubWarning,
    log::{RepubLog, RepubLogStatus},
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

        /// Error: エラー, プログラムの停止に対する助言
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
                write!(f, "{}\t {}", "Error".on_red().white().bold(), self.as_ref())
            }
        }

        impl Message for RepubError {}
    }

    pub mod warning {
        use super::*;

        /// Warning: 警告, 意図しないプログラムの動作に対する助言
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
                write!(f, "{}\t {}", "Warning".on_yellow().bold(), self.as_ref())
            }
        }

        impl Message for RepubWarning {}
    }

    pub mod log {
        use super::*;

        /// Log: 処理の進捗報告など
        pub struct RepubLog(pub RepubLogStatus, pub String);

        impl Display for RepubLog {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let RepubLog(status, string) = &self;
                write!(f,"{} {}", status, string)
            }
        }

        impl Message for RepubLog {
            fn print(&self) {
                let RepubLog(status,_) = &self;
                if status == &RepubLogStatus::Published {
                    println!("{}", &self);
                } else {
                    info!("{}", &self);
                }
            }
        }

        /// Logの種類(作業の進み具合)
        #[derive(Debug, PartialEq)]
        pub enum RepubLogStatus {
            /// static なファイル(css含む)をtmp_dirに格納した
            Packed,
            /// 変換が必要なファイルを変換してtmp_dirに格納した
            Converted,
            /// ファイルを変換して Zip アーカイブに追加した
            Zipped,
            /// EPUB の作成が完了した
            Published,
            /// tmp_dir を削除した
            Removed,
        }

        impl Display for RepubLogStatus {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                use colored::*;

                let preamble = match &self {
                    RepubLogStatus::Packed => {
                        format!("{:?}", &self).as_str().green().bold()
                    }
                    RepubLogStatus::Converted => {
                        format!("{:?}", &self).as_str().bright_green().bold()
                    }
                    RepubLogStatus::Zipped => {
                        format!("{:?}", &self).as_str().purple().bold()
                    }
                    RepubLogStatus::Published => {
                        format!("{:?}", &self).as_str().blue().bold()
                    }
                    RepubLogStatus::Removed => {
                        format!("{:?}", &self).as_str().yellow().bold()
                    }
                };

                write!(f, "[{}]\t", &preamble)
            }
        }

        #[test]
        fn test() {
            println!("{:?}", RepubLogStatus::Packed);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    extern crate env_logger;

    fn init() {
        std::env::set_var("RUST_LOG", "info");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn log() {
        init();

        info!("info log");
        RepubLog(RepubLogStatus::Packed, String::from("Packed")).print();
        RepubLog(RepubLogStatus::Converted, String::from("Converted")).print();
        RepubLog(RepubLogStatus::Zipped, String::from("Zipped")).print();
        RepubLog(RepubLogStatus::Published, String::from("Published")).print();
        RepubLog(RepubLogStatus::Removed, String::from("Removed")).print();

        RepubWarning(String::from("Warning")).print();
        RepubError(String::from("Error")).print();
    }
}