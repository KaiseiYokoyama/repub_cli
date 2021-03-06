use crate::{prelude::*, tmpfile::*, load::*, data::*, toc::*};
use media_type::*;
pub use properties::*;

pub struct Composer {
    tmp_dir: TmpDir,
    data: InputData,
    composed: Composed,
    toc: TableOfContents,
    navigation: Option<ComposedItem>,
}

impl TryFrom<InputData> for Composer {
    type Error = failure::Error;

    fn try_from(value: InputData) -> Result<Self, Self::Error> {
        let tmp_dir = TmpDir::new()?;
        let composed = Composed::new();

        Ok(Self {
            tmp_dir,
            data: value,
            composed,
            toc: TableOfContents::new(),
            navigation: None,
        })
    }
}

impl Drop for Composer {
    fn drop(&mut self) {
        fn output_cfg(cfg: &Config) -> RepubResult<()> {
            // 設定ファイルの出力
            if cfg.config {
                let path = {
                    let path = &cfg.target;
                    if path.is_file() {
                        path.with_file_name(CONFIG_JSON)
                    } else if path.is_dir() {
                        path.join(CONFIG_JSON)
                    } else {
                        unreachable!()
                    }
                };

                let json_str = serde_json::to_string(cfg)?;

                let mut file = match std::fs::File::create(&path) {
                    Ok(file) => file,
                    Err(_) => {
                        std::fs::remove_file(&path)?;
                        std::fs::File::create(&path)?
                    }
                };

                file.write_all(json_str.as_bytes())?;
                file.flush()?;

                RepubLog::config(&format!("Saved to {:?}", &path)).print();
            }

            Ok(())
        }

        // 一時ファイルの削除
        if (!self.data.cfg.save) && cfg!(target_os = "macos") {
            std::fs::remove_dir_all(&self.tmp_dir.path);
            RepubLog::removed(&format!("Temporary files: {:?}", &self.tmp_dir.path)).print();
        }

        if let Err(e) = output_cfg(&self.data.cfg) {
            RepubError(format!("{}", e)).print();
        }
    }
}

impl Composer {
    fn filter_ignored_source<T: AsRef<Source>>(src: Vec<T>, cfg: &Config) -> Vec<T> {
        let ignores = &cfg.ignores;
        src.into_iter().filter(|c| {
            if let Some(p) = PathBuf::path_diff(&cfg.target, &c.as_ref().path) {
                !ignores.contains(&p)
            } else { false }
        }).collect::<Vec<T>>()
    }

    /// css を tmp directoryに格納する
    pub fn compose_css(&mut self) -> RepubResult<&mut Self> {
        // ignore する
        let style_files = Self::filter_ignored_source(self.data.files.style_files.clone(), &self.data.cfg);

        for file in &style_files {
            let relative_path = PathBuf::path_diff(&self.data.cfg.target, &file.path).unwrap();
            let to = self.tmp_dir.oebps.path.join(&relative_path);

            // epub3の対応している拡張子かどうかを確認する -> そうでなければreturn
            let composed = ComposedItem::new(file, &to, "css", self.composed.style_items.len())?;
            // 対応している拡張子ならばcopy
            std::fs::copy(&file.path, &to)?;
            // ログ出力
            RepubLog::packed(&format!("{:?}", &relative_path)).print();

            self.composed.style_items.push(composed);
        }

        Ok(self)
    }

    /// static file を tmp directory に格納する
    pub fn compose_static(&mut self) -> RepubResult<&mut Self> {
        // ignore する
        let static_files = Self::filter_ignored_source(self.data.files.static_files.clone(), &self.data.cfg);

        for file in &static_files {
            let relative_path = PathBuf::path_diff(&self.data.cfg.target, &file.path).unwrap();
            let to = self.tmp_dir.oebps.path.join(&relative_path);

            // epub3の対応している拡張子かどうかを確認する -> そうでなければreturn
            match ComposedItem::new(file, &to, "static", self.composed.static_items.len()) {
                Ok(composed) => {
                    // 対応している拡張子ならばcopy
                    std::fs::copy(&file.path, &to)?;
                    // ログ出力
                    RepubLog::packed(&format!("{:?}", &relative_path)).print();

                    self.composed.static_items.push(composed);
                }
                Err(e) => {
                    RepubWarning(format!("{:?} : {}", &file.path, &e)).print();
                    continue;
                }
            }
        }

        Ok(self)
    }

    /// content file を変換して, 内容を目次に登録し tmp directory に格納する
    /// `.md`ファイルを変換してつくる`.xhtml`ファイルに`.css`を適用するので,
    /// このメソッドの実行までに`compose_css()`を実行する必要がある
    pub fn compose_contents(&mut self) -> RepubResult<&mut Self> {
        use html5ever::{
            serialize,
            parse_fragment,
            ParseOpts,
            serialize::SerializeOpts,
            QualName,
            LocalName,
            rcdom::{RcDom, NodeData, Handle},
            tendril::{TendrilSink, StrTendril},
            Attribute,
        };

        fn register_to(toc: &mut TableOfContents, xhtml: &String, path_buf: &PathBuf) -> String {
            fn create_attribute(name: &str, value: &str) -> Attribute {
                Attribute {
                    name: QualName::new(None, ns!(), LocalName::from(name)),
                    value: StrTendril::from(value),
                }
            }

            let parser = parse_fragment(
                RcDom::default(),
                ParseOpts::default(),
                QualName::new(None, ns!(html), local_name!("body")),
                vec![],
            );
            let dom = parser.one(xhtml.clone());

            let bind = dom.document.children.borrow();
            let bind2 = bind[0].children.borrow();
            let mut children = bind2.iter();

            while let Some(child) = children.next() {
                match child.data {
                    NodeData::Element {
                        ref name,
                        ref attrs, ..
                    } => {
                        fn node_text(node: &Handle, text: &mut String) {
                            match node.data {
                                NodeData::Text { ref contents, .. } => {
                                    let bind = contents.borrow();
                                    let s = bind.as_ref();
                                    text.push_str(s);
                                }
                                _ => {}
                            }

                            for child in node.children.borrow().iter() {
                                node_text(child, text);
                            }
                        }
                        let level = match name.local {
                            local_name!("h1") => 1,
                            local_name!("h2") => 2,
                            local_name!("h3") => 3,
                            local_name!("h4") => 4,
                            local_name!("h5") => 5,
                            _ => continue,
                        };

                        let id = format!("header{}", toc.size());
                        attrs.borrow_mut().push(create_attribute("id", &id));

                        // タイトル抽出
                        let title = {
                            let mut title = String::new();
                            node_text(child, &mut title);
                            // サニタイズ(テキストと認識されているので, HTMLとして成立していない)
                            title.replace("<", "&lt;").replace(">", "&gt;")
                        };

                        // tocに登録
                        let toc_item = {
                            let path_buf = path_buf.clone();
                            let id = Some(id);

                            ToCItem {
                                items: Vec::new(),
                                path_buf,
                                id,
                                title: title.clone(),
                                level,
                            }
                        };
                        toc.push(Box::new(toc_item));

                        // ログ出力
                        RepubLog::indexed(
                            &format!("{} {} ({})",
                                     "#".repeat(level as usize),
                                     &title,
                                     path_buf.file_name()
                                         .map(|e| e.to_str().unwrap_or_default())
                                         .unwrap_or_default()
                            )).print();
                    }
                    _ => {}
                }
            }

            let mut bytes = vec![];
            serialize(&mut bytes, &dom.document.children.borrow()[0], SerializeOpts::default()).unwrap();
            let xhtml = String::from_utf8(bytes).unwrap();

            // domをhtmlに変換しているので、xhtmlとは文法の合わない箇所がある
            let peaces: Vec<&str> = xhtml.split('<').collect();
            peaces.into_iter().map(|s| {
                if s.starts_with("img") || s.starts_with("br") || s.starts_with("hr") {
                    s.replacen(">", " />", 1)
                } else { s.to_string() }
            }).collect::<Vec<String>>().join("<")
        }

        fn convert_content_file(file: &ContentSource, slf: &mut Composer, styles: Option<Vec<ComposedItem>>) -> RepubResult<ComposedItem> {
            match file.convert_type {
                ConvertType::MarkdownToXHTML => {
                    let relative_path = PathBuf::path_diff(&slf.data.cfg.target, &file.src.path).unwrap();
                    let to = {
                        let mut to_xhtml = slf.tmp_dir.oebps.path.join(&relative_path);
                        to_xhtml.set_extension("xhtml");
                        to_xhtml
                    };

                    let xhtml = {
                        let mut options = comrak::ComrakOptions::default();
                        options.github_pre_lang = true;
                        options.ext_strikethrough = true;
                        options.ext_tagfilter = true;
                        options.ext_table = true;
                        options.ext_autolink = true;
                        options.ext_tasklist = true;
                        options.hardbreaks = true;

                        let source_str = {
                            let mut string = String::new();
                            std::fs::File::open(&file.src.path)?.read_to_string(&mut string)?;
                            string
                        };

                        comrak::markdown_to_html(&source_str, &options)
                    };

                    // tocに登録, 整形
                    let xhtml = register_to(&mut slf.toc, &xhtml, &to);

                    // スタイルシートへの<link>要素を生成
                    let style_xhtml = if let Some(styles) = styles {
                        styles.iter()
                            .map(|ci| {
                                let rel_path
                                    = PathBuf::path_diff(&to, &ci.path)
                                    .unwrap();
                                format!("<link type=\"text/css\" rel=\"stylesheet\" href=\"{}\" />", &rel_path.to_str().unwrap())
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    } else { slf.composed.styles_links(&to) };

                    // xhtmlを生成
                    let xhtml = format!(
                        include_str!("literals/template.xhtml"),
                        &style_xhtml,
                        &file.src.file_name,
                        &xhtml
                    );

                    // 書き込み
                    std::fs::File::create(&to)?.write_all(xhtml.as_bytes())?;

                    // ログ出力
                    RepubLog::converted(&format!("{:?}", relative_path)).print();

                    ComposedItem::new(&file.src, &to, "contents", slf.composed.contents.len())
                }
                ConvertType::NoConversion => {
                    let relative_path = PathBuf::path_diff(&slf.data.cfg.target, &file.src.path).unwrap();
                    let to = slf.tmp_dir.oebps.path.join(&relative_path);

                    // 書き込み
                    std::fs::copy(&file.src.path, &to)?;

                    // ログ出力
                    RepubLog::packed(&format!("{:?}", relative_path)).print();

                    ComposedItem::new(&file.src, &to, "contents", slf.composed.contents.len())
                }
            }
        }

        let srcs = if let Some(srcs) = self.data.cfg.sequence.clone().map(|s| s.ptc(&self.data.cfg)) {
            srcs
        } else {
            self.data.files.content_files.clone()
        };

        let ignores = self.data.cfg.ignores.clone().ptc(&self.data.cfg);

        // srcs から ignore に該当するものを差し引く
        let srcs = srcs.into_iter().filter(|p| !ignores.contains(p)).collect::<Vec<ContentSource>>();

        for src in &srcs {
            let (properties, styles) =
                if let Some(ContentConfigure { properties, styles, .. }) = self.data.cfg.config(src.as_ref()) {
                    let properties = properties.iter().map(|p| p.clone()).collect::<Vec<Properties>>();

                    let styles = styles.iter().map(|p| {
                        let path = self.tmp_dir.oebps.path.join(p);
                        self.composed.style_items.iter().find(|s| s.path == path).cloned()
                    }).flat_map(|c| c).collect::<Vec<ComposedItem>>();

                    (Some(properties), Some(styles))
                } else { (None, None) };

            let mut composed = convert_content_file(src, self, styles)?;

            if let Some(mut prop) = properties {
                composed.properties.append(&mut prop);
            }
            self.composed.contents.push(composed);
        }

        Ok(self)
    }

    /// cover image が存在すれば pack する
    pub fn compose_cover_image(&mut self) -> RepubResult<&mut Self> {
        if let Some(image) = &self.data.cfg.cover_image {
            let path = &self.data.cfg.target.join(image);
            let relative_path = image;
            let to = self.tmp_dir.oebps.path.join(&relative_path);

            // epub3の対応している拡張子かどうかを確認する
            match ComposedItem::without_src(&to, "static", self.composed.static_items.len()) {
                Ok(mut composed) => {
                    // 対応している拡張子ならばcopy
                    std::fs::copy(&path, &to)?;
                    // ログ出力
                    RepubLog::packed(&format!("Cover Image ({:?})", &relative_path)).print();

                    composed.properties.push(Properties::CoverImage);
                    // <spine>要素への登録は不要 -> 登録先はstatic_itemsでok
                    self.composed.static_items.push(composed);
                }
                Err(e) => {
                    RepubWarning(format!("{:?} : {}", &path, &e)).print();
                }
            }
        }

        Ok(self)
    }

    /// self.toc を参照して, navigation.xhtml を生成する
    /// compose_css -> compose_static -> compose_contents -> *compose_nav* -> compose_opf
    pub fn compose_nav(&mut self) -> RepubResult<&mut Self> {
        let path = self.tmp_dir.oebps.path.join("navigation.xhtml");

        // todo configによる目次タイトルの変更
        let h1_title = "目次";

        // スタイルシートへの<link>要素を生成
        let style_xhtml = self.composed.styles_links(&path);

        // 目次要素を生成
        let toc = self.toc.to_xhtml(self.data.cfg.toc_depth, &path);

        let xhtml = format!(
            include_str!("literals/navigation.xhtml"),
            h1_title,
            style_xhtml,
            h1_title,
            toc
        );

        std::fs::File::create(&path)?.write_all(xhtml.as_bytes())?;

        // 登録
        let mut composed = ComposedItem::without_src(&path, "navigation", 0)?;
        composed.properties.push(Properties::Nav);
//        self.composed.contents.push(composed);
        self.navigation = Some(composed);

        // ログ出力
        RepubLog::packed(&format!("{:?}", PathBuf::path_diff(&self.tmp_dir.path, &path).unwrap())).print();

        Ok(self)
    }

    /// self.composed を参照して, package.opf を生成する
    /// compose_css -> compose_static -> compose_contents -> compose_nav -> *compose_opf*
    pub fn compose_opf(&mut self) -> RepubResult<&mut Self> {
        use chrono::prelude::*;

        let path = self.tmp_dir.oebps.path.join("package.opf");

        let metadata = format!(
            include_str!("literals/package/metadata"),
            title = self.data.cfg.title.clone(),
            language = self.data.cfg.language.clone(),
            creator = self.data.cfg.creator.clone(),
            book_id = self.data.cfg.book_id.clone(),
            last_mod = Utc::now()
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
                .replace("\"", ""),
        );

        let manifest_str = {
            let items_str
                = self.navigation.iter()
                .chain(self.composed.contents.iter())
                .chain(self.composed.style_items.iter())
                .chain(self.composed.static_items.iter())
                .map(|ci| ci.as_manifest_item(&path))
                .collect::<Vec<String>>()
                .join("\n");

            format!(
                include_str!("literals/package/manifest"),
                items_str = items_str,
            )
        };

        // 並びの変更
        let spine_str = {
            let (handmade_navs, mut contents_without_navs): (Vec<ComposedItem>, Vec<ComposedItem>)
                = self.composed.contents.clone().into_iter()
                .partition(|c| c.properties.contains(&Properties::Nav));

            // sort
            contents_without_navs
                .sort_by(|a, b| a.id.cmp(&b.id));

            let items_str
                = self.navigation.iter()
                .chain(handmade_navs.iter())
                .chain(contents_without_navs.iter())
                .map(|ci| ci.as_spine_item())
                .collect::<Vec<String>>()
                .join("\n");

            format!(
                include_str!("literals/package/spine"),
                ppd = PageProgressionDirection::from(&self.data.cfg.writing_mode).to_string(),
                items_str = items_str,
            )
        };

        let xhtml = format!(
            include_str!("literals/package/package.opf"),
            &metadata,
            &manifest_str,
            &spine_str
        );

        // 書き込み
        std::fs::File::create(&path)?.write_all(xhtml.as_bytes())?;

        // zippingに備えてpathbufを保存
        let package_opf = Some(path.clone());
        self.tmp_dir.oebps.package_opf = package_opf;

        // ログ出力
        RepubLog::packed(&format!("{:?}", PathBuf::path_diff(&self.tmp_dir.path, &path).unwrap())).print();

        Ok(self)
    }

    /// すべてのファイルを(必要があれば)変換, 書き換えをして tmp directory に格納する
    pub fn compose(&mut self) -> RepubResult<()> {
        self.compose_css()?
            .compose_static()?
            .compose_contents()?
            .compose_cover_image()?
            .compose_nav()?
            .compose_opf()?;

        if cfg!(target_os = "macos") {
            self.zip()?;
        }

        Ok(())
    }

    pub fn zip(&mut self) -> RepubResult<()> {
        use zip::{CompressionMethod, write::{FileOptions, ZipWriter}};

        let epub_path = PathBuf::from(&format!("{}.epub", &self.data.cfg.title.clone()));
        let epub = match std::fs::File::create(&epub_path) {
            Ok(file) => {
                file
            }
            Err(_) => {
                std::fs::remove_file(&epub_path)?;
                std::fs::File::create(&epub_path)?
            }
        };

        let mut writer = ZipWriter::new(epub);

        fn write_file(slf: &mut Composer, writer: &mut ZipWriter<std::fs::File>, path: &PathBuf, compression_method: Option<CompressionMethod>) -> RepubResult<()> {
            let rel_path = PathBuf::path_diff(&slf.tmp_dir.path, path).unwrap();

            writer.start_file(rel_path.to_str().unwrap(),
                              FileOptions::default().compression_method(
                                  if let Some(method) = compression_method {
                                      method
                                  } else { CompressionMethod::Deflated }))?;

            let mut file = std::fs::File::open(&path)?;
            let mut bytes = vec![];
            file.read_to_end(&mut bytes)?;

            writer.write_all(bytes.as_slice())?;
            writer.flush()?;

            // ログ出力
            RepubLog::zipped(&format!("{:?}", &rel_path)).print();

            Ok(())
        }

        fn write_dir(slf: &mut Composer, writer: &mut ZipWriter<std::fs::File>, path: &PathBuf) -> RepubResult<()> {
            let rel_path = PathBuf::path_diff(&slf.tmp_dir.path, path).unwrap();

            writer.add_directory_from_path(rel_path.as_path(),
                                           FileOptions::default().compression_method(CompressionMethod::Deflated))?;

            let mut dirs = vec![];

            for entry in std::fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_file() {
                    write_file(slf, writer, &path, None)?;
                } else {
                    // directory の処理は, 直下の file の処理が終わったあと
                    dirs.push(path);
                }
            }

            for dir in dirs {
                write_dir(slf, writer, &dir)?;
            }

            Ok(())
        }

        // mimetype 書き込み
        let Mimetype(mimetype) = self.tmp_dir.mimetype.clone();
        write_file(self, &mut writer, &mimetype, Some(CompressionMethod::Stored))?;

        // META-INF 書き込み
        let MetaInf(meta_inf) = self.tmp_dir.meta_inf.clone();
        write_dir(self, &mut writer, &meta_inf)?;

        // OEBPS 書き込み
        let oebps = self.tmp_dir.oebps.path.clone();
        write_dir(self, &mut writer, &oebps)?;

        writer.finish()?;

        // ログ出力
        RepubLog::published(&format!("{:?}", &epub_path)).print();

        Ok(())
    }
}

struct Composed {
    contents: Vec<ComposedItem>,
    style_items: Vec<ComposedItem>,
    static_items: Vec<ComposedItem>,
}

impl Composed {
    pub fn new() -> Self {
        Self {
            contents: Vec::new(),
            style_items: Vec::new(),
            static_items: Vec::new(),
        }
    }

    pub fn styles_links(&self, content_path: &PathBuf) -> String {
        self.style_items
            .iter()
            .map(|ci| {
                let rel_path
                    = PathBuf::path_diff(content_path, &ci.path)
                    .unwrap();
                format!("<link type=\"text/css\" rel=\"stylesheet\" href=\"{}\" />", &rel_path.to_str().unwrap())
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Clone, Debug)]
struct ComposedItem {
    #[allow(dead_code)]
    src: Option<Source>,
    path: PathBuf,
    id: String,
    media_type: MediaType,
    properties: Vec<Properties>,
}

impl ComposedItem {
    fn new(src: &Source, path: &PathBuf, namespace: &str, len: usize) -> RepubResult<Self> {
        let media_type = MediaType::try_from(path)?;
        let id = format!("{}{}", namespace, len);

        Ok(Self {
            src: Some(src.clone()),
            path: path.clone(),
            id,
            media_type,
            properties: Vec::new(),
        })
    }

    fn without_src(path: &PathBuf, namespace: &str, len: usize) -> RepubResult<Self> {
        let media_type = MediaType::try_from(path)?;
        let id = format!("{}{}", namespace, len);

        Ok(Self {
            src: None,
            path: path.clone(),
            id,
            media_type,
            properties: Vec::new(),
        })
    }

    fn as_manifest_item(&self, opf_path: &PathBuf) -> String {
        let properties_str = if self.properties.is_empty() {
            "".to_string()
        } else {
            format!(
                " properties=\"{}\"",
                self.properties
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        };

        let href = PathBuf::path_diff(opf_path, &self.path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        format!(
            "<item id=\"{}\" href=\"{}\" media-type=\"{}\"{} />",
            &self.id,
            href,
            &self.media_type.to_string(),
            &properties_str
        )
    }

    fn as_spine_item(&self) -> String {
        format!("<itemref idref=\"{}\" />", &self.id)
    }
}

pub mod media_type {
    use super::*;

    /// https://imagedrive.github.io/spec/epub30-publications.xhtml#tbl-core-media-types
    #[derive(Clone, PartialEq, Debug)]
    pub enum MediaType {
        Image(ImageType),
        Application(ApplicationType),
        Audio(AudioType),
        Text(TextType),
    }

    impl FromStr for MediaType {
        type Err = failure::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Option::None
                .or(
                    ImageType::from_str(s).ok().map(|t| MediaType::Image(t))
                )
                .or(
                    ApplicationType::from_str(s).ok().map(|t| MediaType::Application(t))
                )
                .or(
                    AudioType::from_str(s).ok().map(|t| MediaType::Audio(t))
                )
                .or(
                    TextType::from_str(s).ok().map(|t| MediaType::Text(t))
                )
                .ok_or(format_err!("EPUB3は拡張子 {} に対応していません",s))
        }
    }

    impl Default for MediaType {
        fn default() -> Self {
            MediaType::Application(ApplicationType::XHTML)
        }
    }

    impl ToString for MediaType {
        fn to_string(&self) -> String {
            match self {
                MediaType::Image(t) => format!("image/{}", &t.to_string()),
                MediaType::Application(t) => format!("application/{}", &t.to_string()),
                MediaType::Audio(t) => format!("audio/{}", &t.to_string()),
                MediaType::Text(t) => format!("text/{}", t.to_string()),
            }
        }
    }

    impl TryFrom<&PathBuf> for MediaType {
        type Error = failure::Error;

        fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
            let ext = value.extension().ok_or(
                format_err!("Failed to unwrap Option<&OsStr> on {}:{}:{}",file!(),line!(),column!())
            )?.to_str().ok_or(
                format_err!("Failed to unwrap Option<&str> on {}:{}:{}",file!(),line!(),column!())
            )?;

            Ok(MediaType::from_str(&ext)?)
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum ImageType {
        GIF,
        JPEG,
        PNG,
        SVG,
    }

    impl FromStr for ImageType {
        type Err = failure::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "gif" => Ok(ImageType::GIF),
                "jpeg" | "jpg" | "jpe" => Ok(ImageType::JPEG),
                "png" => Ok(ImageType::PNG),
                "svg" | "svgz" => Ok(ImageType::SVG),
                _ =>
                    Err(format_err!("Warning: .{} is not in EPUB Core Media Types", s))
            }
        }
    }

    impl ToString for ImageType {
        fn to_string(&self) -> String {
            match self {
                ImageType::GIF => "gif",
                ImageType::JPEG => "jpeg",
                ImageType::PNG => "png",
                ImageType::SVG => "svg+xml",
            }.to_string()
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum ApplicationType {
        /// XHTML Content Document と EPUB Navigation Document
        XHTML,
        /// OpenType Font
        OpenType,
        /// WOFF Font
        WOFF,
        /// EPUB Media Overlay Document
        MediaOverlays,
        /// Text-to-Speech (TTS) 発音語彙
        PLS,
    }

    impl FromStr for ApplicationType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "xhtml" | "xht" => Ok(ApplicationType::XHTML),
                "otf" | "otc" | "ttf" | "ttc" => Ok(ApplicationType::OpenType),
                "woff" | "woff2" => Ok(ApplicationType::WOFF),
                "smil" => Ok(ApplicationType::MediaOverlays),
                "pls" => Ok(ApplicationType::PLS),
                _ => Err(())
            }
        }
    }

    impl ToString for ApplicationType {
        fn to_string(&self) -> String {
            match self {
                ApplicationType::XHTML => "xhtml+xml",
                ApplicationType::OpenType => "vnd.ms-opentype",
                ApplicationType::WOFF => "font-woff",
                ApplicationType::MediaOverlays => "smil+xml",
                ApplicationType::PLS => "pls+xml",
            }.to_string()
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum AudioType {
        /// MP3 オーディオ
        MPEG,
        /// MP4 コンテナを使用している AAC LC オーディオ
        MP4,
    }

    impl FromStr for AudioType {
        type Err = failure::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "mp3" => Ok(AudioType::MPEG),
                "aac" | "mp4" => Ok(AudioType::MP4),
                _ =>
                    Err(format_err!("Warning: .{} is not in EPUB Core Media Types", s))
            }
        }
    }

    impl ToString for AudioType {
        fn to_string(&self) -> String {
            match self {
                AudioType::MPEG => "mpeg",
                AudioType::MP4 => "mp4",
            }.to_string()
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum TextType {
        CSS,
        JS,
    }

    impl FromStr for TextType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "css" => Ok(TextType::CSS),
                "js" => Ok(TextType::JS),
                _ => Err(())
            }
        }
    }

    impl ToString for TextType {
        fn to_string(&self) -> String {
            match self {
                TextType::CSS => "css",
                TextType::JS => "javascript",
            }.to_string()
        }
    }
}

pub mod properties {
    use super::*;

    /// https://imagedrive.github.io/spec/epub30-publications.xhtml#sec-item-property-values
    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    #[allow(dead_code)]
    #[serde(rename_all = "kebab-case")]
    pub enum Properties {
        /// cover-image プロパティは、出版物のカバーイメージとして説明され Publication Resource を識別する
        CoverImage,
        /// mathml プロパティは Publication Resource に1つまたは複数の MathML マークアップのインスタンスが含まれる場合に記述する
        MathML,
        /// nav プロパティは出版物の EPUB Navigation Document を構成する Publication Resource が記述されていることを示している
        Nav,
        /// remote-resources プロパティは EPUB Container の外部にある他の Publication Resource を参照している一つ以上の Publication Resource が記述されていることを示している
        RemoteResources,
        /// scripted プロパティは Publication Resource に記述された Scripted Content Document（すなわち、HTML5 forms からスクリプト化されたコンテンツや要素を含む）を示している
        Scripted,
        /// svg プロパティが記述された Publication Resource は一つ以上の SVG マークアップインスタンスが含まれていることを示している
        Svg,
        /// switch プロパティが記述された Publication Resource は一つ以上の epub:switch 要素インスタンスが含まれていることを示している
        Switch,
    }

    impl ToString for Properties {
        fn to_string(&self) -> String {
            match self {
                Properties::CoverImage => "cover-image",
                Properties::MathML => "mathml",
                Properties::Nav => "nav",
                Properties::RemoteResources => "remote-resources",
                Properties::Scripted => "scripted",
                Properties::Svg => "svg",
                Properties::Switch => "switch"
            }.to_string()
        }
    }
}

use std::path::Path;
use crate::prelude::PathBuf;

trait PathToContentFile {
    fn ptc(&self, cfg: &Config) -> Vec<ContentSource>;
}

impl<T: AsRef<Path>> PathToContentFile for Vec<T> {
    fn ptc(&self, cfg: &Config) -> Vec<ContentSource> {
        self.iter().map(|p| {
            let path = cfg.target.join(p);
            let src = Source::try_from(&path)?;

            ContentSource::try_from(src)
        }).flat_map(|c| c).collect::<Vec<ContentSource>>()
    }
}

/// exec with --nocapture
#[test]
fn test_html5ever() {
    use html5ever::{
        serialize,
        parse_fragment,
        ParseOpts,
        serialize::SerializeOpts,
        QualName,
        LocalName,
        rcdom::{RcDom, NodeData},
        tendril::{TendrilSink, StrTendril},
        Attribute,
    };

    fn create_attribute(name: &str, value: &str) -> Attribute {
        Attribute {
            name: QualName::new(None, ns!(), LocalName::from(name)),
            value: StrTendril::from(value),
        }
    }

    let html_data = "\
<ol>
    <li>
        <a href=\"IIIF_Images.xhtml#header-iiifのマニフェストからimageの一覧を引き出す\">IIIFのマニフェストからImageの一覧を引き出す</a>
    </li>
    <li>
        <a href=\"IIIF_Images.xhtml#header-知見\">知見</a>
        <ol>
            <li>
                <a href=\"IIIF_Images.xhtml#header-rustにおける実行時引数の取得\">Rustにおける実行時引数の取得</a>
            </li>
            <li>
                <a href=\"IIIF_Images.xhtml#header-serde_json\">serde_json</a>
                <ol hidden=\"hidden\">
                    <li>
                        <a href=\"IIIF_Images.xhtml#header-フィールドのrename\">フィールドのRename</a>
                    </li>
                    <li>
                        <a href=\"IIIF_Images.xhtml#header-deserializejson---struct時に余計なフィールドを無視する\">Deserialize(JSON -> Struct)時に、余計なフィールドを無視する</a>
                    </li>
                </ol>
            </li>
        </ol>
    </li>
</ol>
    ";
//    let parser = parse_document(RcDom::default(), ParseOpts::default());
    let parser = parse_fragment(
        RcDom::default(),
        ParseOpts::default(),
        QualName::new(None, ns!(html), local_name!("body")),
        vec![],
    );
    let dom = parser.one(html_data);

    for child in dom.document.children.borrow()[0].children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                if name.local == local_name!("ol") {
                    attrs.borrow_mut().push(create_attribute("id", "new_id"));
                }
            }
            _ => {}
        }
    }

//    let mut bytes = vec![];
//    serialize(&mut bytes, &dom.document, SerializeOpts::default()).unwrap();
//    println!("{}", String::from_utf8(bytes).unwrap());

    let mut bytes = vec![];
    serialize(&mut bytes, &dom.document.children.borrow()[0], SerializeOpts::default()).unwrap();
    println!("{}", String::from_utf8(bytes).unwrap());

    assert_eq!(1, 1)
}

/// exec with --nocapture
#[test]
fn test_print_failure_err() {
    println!("{}", format_err!("This is err!"));
}

// exec with --nocapture
//#[test]
//fn scraper_edit_mem_replace() {
//    use scraper::*;
//
//    let html = r#"
//    <ul>
//        <li id="1">Foo</li>
//        <li id="2">Bar</li>
//        <li id="3">Baz</li>
//    </ul>
//"#;
//
//    let fragment = Html::parse_fragment(html);
//    let selector = Selector::parse("li").unwrap();
//
//    for element in fragment.select(&selector) {
//        match element.value().id() {
//            Some(mut s) => {
//                let old = std::mem::replace(&mut s,"new_id");
//                println!("id replaced: {} => {:?}",old,element.value().id());
//            }
//            _ => {}
//        }
//    }
//}
