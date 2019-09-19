pub use message::{
    Message,
    error::RepubError,
    warning::RepubWarning,
    log::{RepubLog, RepubLogStatus},
};

use crate::prelude::*;

pub const CONFIG_JSON :&str = "repub_config.json";

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
                use colored_truecolor::*;
                write!(f, "{}\t {}", "[Error]".on_red().white().bold(), self.as_ref())
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
                use colored_truecolor::*;
                write!(f, "{}\t {}", "[Warning]".on_yellow().bold(), self.as_ref())
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
                write!(f, "{} {}", status, string)
            }
        }

        impl Message for RepubLog {
            fn print(&self) {
                let RepubLog(status, _) = &self;
                if status == &RepubLogStatus::Published || status == &RepubLogStatus::Config {
                    println!("{}", &self);
                } else {
                    info!("{}", &self);
                }
            }
        }

        impl RepubLog {
            pub fn ignored<T: ToString>(to_string: &T) -> Self {
            Self(RepubLogStatus::Ignored, to_string.to_string())
            }

            pub fn packed<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Packed, to_string.to_string())
            }

            pub fn converted<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Converted, to_string.to_string())
            }

            pub fn indexed<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Indexed, to_string.to_string())
            }

            pub fn zipped<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Zipped, to_string.to_string())
            }

            pub fn published<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Published, to_string.to_string())
            }

            pub fn removed<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Removed, to_string.to_string())
            }

            pub fn config<T: ToString>(to_string: &T) -> Self {
                Self(RepubLogStatus::Config, to_string.to_string())
            }

            #[allow(dead_code)]
            pub fn custom<T: ToString>(hex: u64, preamble: &T, to_string: &T) -> Self {
                Self(RepubLogStatus::Custom(hex, preamble.to_string()), to_string.to_string())
            }
        }

        /// Logの種類(作業の進み具合)
        #[derive(Debug, PartialEq)]
        pub enum RepubLogStatus {
            /// ignores に含まれるファイルを無視した
            Ignored,
            /// static なファイル(css含む)をtmp_dirに格納した
            Packed,
            /// 変換が必要なファイルを変換してtmp_dirに格納した
            Converted,
            /// ヘッダーを目次に追加した
            Indexed,
            /// ファイルを変換して Zip アーカイブに追加した
            Zipped,
            /// EPUB の作成が完了した
            Published,
            /// tmp_dir を削除した
            Removed,
            /// config を保存した
            Config,
            #[allow(dead_code)]
            Custom(u64, String),
        }

        impl Display for RepubLogStatus {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                use colored_truecolor::*;

                let preamble = match &self {
                    RepubLogStatus::Ignored => {
                        // #9e9e9e grey
                        format!("🙈{:?}", &self).as_str().hex_color(0x9e9e9e).bold()
                    }
                    RepubLogStatus::Packed => {
                        // #4caf50 green
                        format!("📦{:?}", &self).as_str().hex_color(0x4caf50).bold()
                    }
                    RepubLogStatus::Converted => {
                        // #8bc34a light-green
                        format!("📜{:?}", &self).as_str().hex_color(0x8bc34a).bold()
                    }
                    RepubLogStatus::Indexed => {
                        // #009688 teal
                        format!("🏷{:?}", &self).as_str().hex_color(0x009688).bold()
                    }
                    RepubLogStatus::Zipped => {
                        // #827717 lime darken-4
                        format!("🗄{:?}", &self).as_str().hex_color(0x827717).bold()
                    }
                    RepubLogStatus::Published => {
                        // #03a9f4 light-blue
                        format!("📚{:?}", &self).as_str().hex_color(0x03a9f4).bold()
                    }
                    RepubLogStatus::Removed => {
                        // #3f51b5 indigo
                        format!("🗑{:?}", &self).as_str().hex_color(0x3f51b5).bold()
                    }
                    RepubLogStatus::Config => {
                        // #9c27b0 purple
                        format!("🔨{:?}", &self).as_str().hex_color(0x9c27b0).bold()
                    }
                    RepubLogStatus::Custom(hex, string) => {
                        format!("{}", string).as_str().hex_color(hex.clone()).bold()
                    }
                };

                write!(f, "{:<10}\t", &preamble)
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