use crate::prelude::*;

pub use meta_inf::MetaInf;
pub use oebps::OEBPS;
pub use mimetype::Mimetype;

const TMP_DIR_PATH_STR: &str = "repub_tmp";

pub struct TmpDir {
    /// 一時ディレクトリのpath
    pub path: PathBuf,
    /// META-INF directory
    pub meta_inf: MetaInf,
    /// OEBPS directory
    pub oebps: OEBPS,
    /// mimetype
    pub mimetype: Mimetype,
}

impl TmpDir {
    pub fn new() -> RepubResult<Self> {
        let path = PathBuf::from(TMP_DIR_PATH_STR);
        std::fs::create_dir_all(&path)?;

        let meta_inf = MetaInf::new(&path)?;
        let oebps = OEBPS::new(&path)?;
        let mimetype = Mimetype::new(&path)?;

        Ok(
            Self {
                path,
                meta_inf,
                oebps,
                mimetype,
            }
        )
    }
}

mod meta_inf {
    use super::*;
    use std::fs::File;

    pub struct MetaInf;

    impl MetaInf {
        pub fn new(tmpdir_path: &PathBuf) -> RepubResult<Self> {
            let path = tmpdir_path.join("META-INF");
            std::fs::create_dir_all(&path)?;

            // container.xmlを書き込み
            let container_xml = path.join("container.xml");
            File::open(container_xml)?.write_all(include_str!("literals/container.xml").as_bytes())?;

            Ok(Self)
        }
    }
}

mod oebps {
    use super::*;

    pub struct OEBPS {
        /// OEBPS directory の path
        pub path: PathBuf,
    }

    impl OEBPS {
        pub fn new(tmpdir_path: &PathBuf) -> RepubResult<Self> {
            let path = tmpdir_path.join("OEBPS");
            std::fs::create_dir_all(&path)?;

            Ok(Self { path })
        }
    }
}

mod mimetype {
    use super::*;
    use std::fs::File;

    pub struct Mimetype;

    impl Mimetype {
        pub fn new(tmpdir_path: &PathBuf) -> RepubResult<Self> {
            let path = tmpdir_path.join("mimetype");

            let mut mimetype = File::create(path)?;
            mimetype.write_all(include_str!("literals/mimetype").as_bytes())?;

            Ok(Self)
        }
    }
}