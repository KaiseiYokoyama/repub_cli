use crate::prelude::*;
use crate::load::*;

use files::Files;

#[derive(Debug)]
pub struct Data {
    cfg: Config,
    files: Files,
}

impl From<Input> for Data {
    fn from(input: Input) -> Self {
        let Input { cfg, src } = input;

        let files = Files::from(src);

        Self {
            cfg,
            files,
        }
    }
}

mod files {
    use super::*;

    #[derive(Debug)]
    pub struct Files {
        /// 変換を必要とするファイル
        content_files: Vec<ContentFile>,
        /// スタイルを指定するファイル
        style_files: Vec<Source>,
        /// その他、変換を必要としないファイル
        static_files: Vec<Source>,
    }

    impl From<Vec<Source>> for Files {
        fn from(srcs: Vec<Source>) -> Self {
            let (mut content_files, mut style_files, mut static_files) = (Vec::new(), Vec::new(), Vec::new());

            for src in srcs {
                if let Ok(content_file) = ContentFile::try_from(src.clone()) {
                    content_files.push(content_file);
                    continue;
                }

                if let Some(ext) = &src.ext {
                    if ext.as_str() == "css" {
                        style_files.push(src);
                        continue;
                    }
                }

                static_files.push(src);
            }

            Self {
                content_files,
                style_files,
                static_files,
            }
        }
    }

    /// コンテンツ
    /// 変換を必要とするファイル
    #[derive(Debug)]
    struct ContentFile {
        src: Source,
        convert_type: ConvertType,
    }

    impl TryFrom<Source> for ContentFile {
        type Error = ();

        fn try_from(value: Source) -> Result<Self, Self::Error> {
            let convert_type = {
                let ext = value.ext.as_ref().ok_or(())?;
                match ext.as_str() {
                    "md" => ConvertType::MarkdownToXHTML,
                    _ => return Err(()),
                }
            };

            Ok(Self {
                src: value,
                convert_type,
            })
        }
    }

    /// 変換の種類
    #[derive(Debug)]
    enum ConvertType {
        MarkdownToXHTML,
    }
}