use clap::ArgMatches;

use crate::prelude::*;
pub use source::Source;
pub use config::{Config, WritingMode, PageProgressionDirection};

/// 入力された情報(設定およびfile)
#[derive(Debug)]
pub struct Input {
    pub cfg: config::Config,
    pub src: Vec<source::Source>,
}

trait ArgMatchesExt {
    fn value_of_or_err(&self, name: &str) -> RepubResult<&str>;
}

impl<'a> ArgMatchesExt for ArgMatches<'a> {
    fn value_of_or_err(&self, name: &str) -> RepubResult<&str> {
        self.value_of(name).ok_or(
            format_err!("引数{}がありません",name)
        )
    }
}

impl<'a> TryFrom<clap::ArgMatches<'a>> for Input {
    type Error = failure::Error;

    fn try_from(value: clap::ArgMatches<'a>) -> Result<Self, Self::Error> {
        let source_path_buf = {
            let source_path_str = value.value_of_or_err("input")?;
            PathBuf::from_str(source_path_str)?
        };

        let src = Source::try_from_path_buf(&source_path_buf)?;

        let cfg = Config::try_from(&value)?;

        Ok(Self {
            src,
            cfg,
        })
    }
}

mod config {
    use super::*;
    pub use writing_mode::{WritingMode, PageProgressionDirection};

    /// 出力設定
    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct Config {
        /// コマンドの<input>として与えられたpath(変換対象)
        pub target: PathBuf,
        /// 書式
        pub writing_mode: WritingMode,
        /// タイトル
        pub title: String,
        /// 著者, 編集者, 翻訳者など
        pub creator: String,
        /// 言語
        pub language: String,
        /// epub形式で本の識別に利用されるid
        pub book_id: String,
        /// 目次に表示するheaderの最低レベル
        /// 1を指定すればh1のみ、5以上を指定すればh1~h5の全てのheaderが目次に表示される
        pub toc_depth: u8,
        /// ログ表示するか否か
        pub verbose: bool,
        /// tmp_dir を消去するか否か
        pub save: bool,
        /// config ファイルを出力する
        pub config: bool,
        /// 表紙
        pub cover_image: Option<PathBuf>,
    }

    impl<'a> TryFrom<&clap::ArgMatches<'a>> for Config {
        type Error = failure::Error;

        fn try_from(value: &clap::ArgMatches<'a>) -> Result<Self, Self::Error> {
            use rand::Rng;
            use rand::distributions::Alphanumeric;

            let target = {
                let source_path_str = value.value_of_or_err("input")?;
                PathBuf::from_str(source_path_str)?
            };

            // 設定ファイル読み込み
            fn from_json(target: &PathBuf) -> Option<Config> {
                if target.is_file() {
                    return None;
                }

                let json_path = target.join(CONFIG_JSON);

                if !json_path.is_file() {
                    return None;
                }

                let json = match std::fs::read_to_string(&json_path) {
                    Ok(json) => json,
                    Err(_) => return None,
                };

                match serde_json::from_str::<Config>(&json) {
                    Ok(cfg) => Some(cfg),
                    Err(e) => {
                        RepubWarning(format!("{:?} {}", &json_path, &e)).print();
                        None
                    }
                }
            }

            let mut cfg = from_json(&target);

            let title = {
                if let Some(title) = value.value_of("title") {
                    title.to_string()
                } else if let Some(cfg) = &cfg {
                    cfg.title.clone()
                } else {
                    print!("Title: ");
                    std::io::stdout().flush().context("Failed to read line.")?;

                    let mut title = String::new();
                    std::io::stdin().read_line(&mut title)
                        .expect("Failed to read line");
                    title.trim().to_string()
                }
            };

            let creator = {
                if let Some(creator) = value.value_of("creator") {
                    creator.to_string()
                } else if let Some(cfg) = &cfg {
                    cfg.creator.clone()
                } else {
                    print!("Creator: ");
                    std::io::stdout().flush().context("Failed to read line.")?;

                    let mut creator = String::new();
                    std::io::stdin().read_line(&mut creator)
                        .expect("Failed to read line");
                    creator.trim().to_string()
                }
            };

            let language = {
                if let Some(language) = value.value_of("language") {
                    language.to_string()
                } else if let Some(cfg) = &cfg {
                    cfg.language.clone()
                } else {
                    print!("Language: ");
                    std::io::stdout().flush().context("Failed to read line.")?;

                    let mut language = String::new();
                    std::io::stdin().read_line(&mut language)
                        .expect("Failed to read line");
                    language.trim().to_string()
                }
            };

            let writing_mode = {
                if let Some(mode) = value.value_of("writing_mode") {
                    WritingMode::from_str(mode)?
                } else if let Some(cfg) = &cfg {
                    cfg.writing_mode.clone()
                } else {
                    WritingMode::default()
                }
            };

            let book_id = {
                if let Some(id) = value.value_of("book_id") {
                    id.to_string()
                } else if let Some(cfg) = &cfg {
                    cfg.book_id.clone()
                } else {
                    rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(30)
                        .collect::<String>()
                }
            };

            let cover_image = {
                if let Some(cover_image) = value.value_of("cover_image") {
                    let path = PathBuf::from(cover_image);

                    // Validation
                    if path.exists() {
                        PathBuf::path_diff(&target, &path)
                    } else { None }
                } else if let Some(cfg) = &cfg {
                    // Validation
                    let cover_image = cfg.cover_image.clone();
                    cover_image.filter(|p| {
                        target.join(p).exists()
                    })
                } else {
                    None
                }
            };

            let toc_depth = {
                if let Some(level) = value.value_of("toc_depth") {
                    match level.parse::<u8>() {
                        Ok(ok) => ok,
                        Err(_) => {
                            let level_alt = 2;
                            RepubWarning(format!("{} は目次の最低レベルに設定できません {} に設定しました", &level, &level_alt)).print();
                            level_alt
                        }
                    }
                } else if let Some(cfg) = &cfg {
                    cfg.toc_depth.clone()
                } else { 2 }
            };

            let verbose = {
                let a = value.is_present("verbose");
                let b =
                    if let Some(cfg) = &cfg {
                        cfg.verbose.clone()
                    } else { false };
                (a || b) && !(a && b)
            };
            if verbose {
                std::env::set_var("RUST_LOG", "info");
            }

            let save = {
                let a = value.is_present("save");
                let b =
                    if let Some(cfg) = &cfg {
                        cfg.save.clone()
                    } else { false };
                (a || b) && !(a && b)
            };

            let config = {
                let a = value.is_present("config");
                let b =
                    if let Some(cfg) = &cfg {
                        cfg.config.clone()
                    } else { false };
                (a || b) && !(a && b)
            };

            // logger を初期化
            env_logger::Builder::from_default_env()
                .format(|buf, record| writeln!(buf, "{}", record.args()))
                .init();

            Ok(Self {
                target,
                writing_mode,
                title,
                creator,
                language,
                book_id,
                toc_depth,
                verbose,
                save,
                config,
                cover_image,
            })
        }
    }

    mod writing_mode {
        use super::*;
        use std::fmt;

        use serde::{Serializer, Deserializer};
        use serde::de::Visitor;

        pub use page_progression_direction::*;

        /// 書式
        ///  [参考](https://developer.mozilla.org/ja/docs/Web/CSS/writing-mode)
        #[derive(Debug, Clone)]
        pub enum WritingMode {
            /// コンテンツは左から右へ水平に、上から下へ垂直方向に流れます。次の水平な行は、前の行の下に配置されます。
            HorizontalTb,
            /// コンテンツは上から下へ垂直に、右から左へ水平方向に流れます。次の垂直な行は、前の行の左に配置されます。
            VerticalRl,
            /// コンテンツは上から下へ垂直に、左から右へ水平方向に流れます。次の垂直な行は、前の行の右に配置されます。
            VerticalLr,
        }

        impl Default for WritingMode {
            fn default() -> Self {
                WritingMode::HorizontalTb
            }
        }

        impl FromStr for WritingMode {
            type Err = failure::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "htb" => Ok(WritingMode::HorizontalTb),
                    "vrl" => Ok(WritingMode::VerticalRl),
                    "vlr" => Ok(WritingMode::VerticalLr),
                    _ => Err(format_err!("Writing Modeには htb/vrl/vlrのいずれかを指定してください"))
                }
            }
        }

        impl ToString for WritingMode {
            fn to_string(&self) -> String {
                match self {
                    WritingMode::HorizontalTb => "htb",
                    WritingMode::VerticalRl => "vrl",
                    WritingMode::VerticalLr => "vlr",
                }.to_string()
            }
        }

        impl Serialize for WritingMode {
            fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
                S: Serializer {
                serializer.serialize_str(&self.to_string())
            }
        }

        struct WritingModeVisitor;

        impl<'de> Visitor<'de> for WritingModeVisitor {
            type Value = WritingMode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Writing Modeには htb/vrl/vlrのいずれかを指定してください")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
                E: serde::de::Error, {
                match WritingMode::from_str(v) {
                    Ok(ok) => Ok(ok),
                    Err(_) => Err(E::custom(format!("Writing Modeには htb/vrl/vlrのいずれかを指定してください")))
                }
            }
        }

        impl<'de> Deserialize<'de> for WritingMode {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
                D: Deserializer<'de> {
                deserializer.deserialize_str(WritingModeVisitor)
            }
        }

        mod page_progression_direction {
            use super::WritingMode;

            pub enum PageProgressionDirection {
                LtR,
                RtL,
                Default,
            }

            impl ToString for PageProgressionDirection {
                fn to_string(&self) -> String {
                    match self {
                        PageProgressionDirection::LtR => "ltr",
                        PageProgressionDirection::RtL => "rtl",
                        PageProgressionDirection::Default => "default",
                    }.to_string()
                }
            }

            impl Default for PageProgressionDirection {
                fn default() -> Self {
                    PageProgressionDirection::Default
                }
            }

            impl From<&WritingMode> for PageProgressionDirection {
                fn from(mode: &WritingMode) -> Self {
                    match mode {
                        WritingMode::HorizontalTb => PageProgressionDirection::Default,
                        WritingMode::VerticalLr => PageProgressionDirection::LtR,
                        WritingMode::VerticalRl => PageProgressionDirection::RtL,
                    }
                }
            }
        }
    }
}

mod source {
    use super::*;
    use std::fs::DirEntry;

    #[derive(Debug, Clone)]
    pub struct Source {
        pub file_name: String,
        pub ext: Option<String>,
        pub path: PathBuf,
    }

    impl Source {
        pub fn try_from_path_buf(value: &PathBuf) -> RepubResult<Vec<Self>> {
            if value.is_file() {
                return Ok(vec![Self::try_from(value)?]);
            }

            let mut entries
                = std::fs::read_dir(value)?
                .flat_map(|e| e)
                .collect::<Vec<DirEntry>>();
            entries.sort_by_key(|e| e.path());

            let mut vec = Vec::new();
            for entry in entries {
                let path = entry.path();

                vec.append(&mut Self::try_from_path_buf(&path)?);
            }

            Ok(vec)
        }
    }

    impl TryFrom<&PathBuf> for Source {
        type Error = failure::Error;

        fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
            use crate::prelude::*;

            if !value.exists() {
                return Err(format_err!("{}",RepubError(format!("{:?}は存在しません", &value))));
            }

            if !value.is_file() {
                return Err(format_err!("{}",RepubError(format!("{:?}はファイルではありません", &value))));
            }

            let file_name = {
                value.file_name().map(|os| os.to_str().unwrap().to_string())
            }.unwrap();

            let ext = {
                value.extension().map(|os| os.to_str().unwrap().to_string())
            };

            Ok(Self {
                file_name,
                ext,
                path: value.clone(),
            })
        }
    }

//    pub enum SourceType {
//        /// 変換の必要なfile ex. .md
//        Content,
//        /// css file
//        Style,
//        /// 変換の不要なfile ex. .png, .jpeg
//        Static,
//    }
//
//    impl FromStr for SourceType {
//        type Err = failure::Error;
//
//        fn from_str(s: &str) -> Result<Self, Self::Err> {
//            Ok(match s {
//                "md" => SourceType::Content,
//                "css" => SourceType::Style,
//                _ => SourceType::Static,
//            })
//        }
//    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn app_to_input() -> RepubResult<()> {
        let app = crate::app::app();
        let _input = Input::try_from(app.get_matches())?;

        Ok(())
    }
}
