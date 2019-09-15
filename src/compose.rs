use crate::{prelude::*, tmpfile::*, load::*, data::*, toc::*};
use media_type::*;
use properties::*;

pub struct Composer {
    tmp_dir: TmpDir,
    data: InputData,
    composed: Composed,
    toc: TableOfContents,
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
        })
    }
}

impl Composer {
    /// css を tmp directoryに格納する
    /// *compose_css* -> compose_static -> compose_contents
    pub fn compose_css(&mut self) -> RepubResult<&mut Self> {
        for file in &self.data.files.style_files {
            let relative_path = PathBuf::path_diff(&self.data.cfg.target, &file.path).unwrap();
            let to = self.tmp_dir.oebps.path.join(&relative_path);

            // epub3の対応している拡張子かどうかを確認する -> そうでなければreturn
            let composed = ComposedItem::new(file, &to, self.composed.style_items.len())?;
            // 対応している拡張子ならばcopy
            std::fs::copy(&file.path, &to)?;
            // todo ログ出力

            self.composed.style_items.push(composed);
        }

        Ok(self)
    }

    /// static file を tmp directory に格納する
    /// compose_css -> *compose_static* -> compose_contents
    pub fn compose_static(&mut self) -> RepubResult<&mut Self> {
        for file in &self.data.files.static_files {
            let relative_path = PathBuf::path_diff(&self.data.cfg.target, &file.path).unwrap();
            let to = self.tmp_dir.oebps.path.join(&relative_path);

            // epub3の対応している拡張子かどうかを確認する -> そうでなければreturn
            let composed = ComposedItem::new(file, &to, self.composed.static_items.len())?;
            // 対応している拡張子ならばcopy
            std::fs::copy(&file.path, &to)?;
            // todo ログ出力

            self.composed.static_items.push(composed);
        }

        Ok(self)
    }

    /// content file を変換して, 内容を目次に登録し tmp directory に格納する
    /// compose_css -> compose_static -> *compose_contents*
    pub fn compose_contents(&mut self) -> RepubResult<&mut Self> {
        use html5ever::{
            serialize,
            parse_fragment,
            parse_document,
            ParseOpts,
            serialize::SerializeOpts,
            QualName,
            LocalName,
            rcdom::{RcDom, NodeData},
            tendril::{TendrilSink, StrTendril},
            Attribute,
        };

        fn register_to(toc: &mut TableOfContents, xhtml: &String, path_buf: &PathBuf) {
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
                        ref attrs,
                        ref template_contents, ..
                    } => {
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
                            if let NodeData::Text { ref contents, .. } = child.children.borrow()[0].data {
                                contents.borrow().to_string()
                            } else {
                                RepubWarning(format!("ヘッダー {} のタイトルを読み込めませんでした", &id)).print();
                                id.clone()
                            }
                        };

                        // tocに登録
                        let toc_item = {
                            let path_buf = path_buf.clone();
                            let id = Some(id);

                            ToCItem {
                                items: Vec::new(),
                                path_buf,
                                id,
                                title,
                                level,
                            }
                        };
                        toc.push(Box::new(toc_item));

                        // todo ログ出力
                    }
                    _ => {}
                }
            }
        }

        for file in &self.data.files.content_files {
            let composed =
                match file.convert_type {
                    ConvertType::MarkdownToXHTML => {
                        let to = {
                            let relative_path = PathBuf::path_diff(&self.data.cfg.target, &file.src.path).unwrap();
                            let mut to_xhtml = self.tmp_dir.oebps.path.join(relative_path);
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

                        // tocに登録
                        register_to(&mut self.toc, &xhtml, &to);

                        // xhtmlを整形
                        let parser = parse_fragment(
                            RcDom::default(),
                            ParseOpts::default(),
                            QualName::new(None, ns!(html), local_name!("body")),
                            vec![],
                        );
                        let dom = parser.one(xhtml);
                        let mut bytes = vec![];
                        serialize(&mut bytes, &dom.document.children.borrow()[0], SerializeOpts::default()).unwrap();
                        let xhtml = String::from_utf8(bytes).unwrap();

                        // スタイルシートへの<link>要素を生成
                        let style_xhtml = self.composed.styles_links(&to);

                        // xhtmlを生成
                        let xhtml = format!(
                            include_str!("literals/template.xhtml"),
                            &style_xhtml,
                            &file.src.file_name,
                            &xhtml
                        );

                        // 書き込み
                        std::fs::File::create(&to)?.write_all(xhtml.as_bytes())?;

                        // todo ログ出力

                        ComposedItem::new(&file.src, &to, self.composed.contents.len())?
                    }
                };

            self.composed.contents.push(composed);
        }

        Ok(self)
    }

    /// すべてのファイルを(必要があれば)変換, 書き換えをして tmp directory に格納する
    pub fn compose(&mut self) -> RepubResult<()> {
        self.compose_css()?.compose_static()?.compose_contents()?;
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

struct ComposedItem {
    src: Source,
    path: PathBuf,
    id: String,
    media_type: MediaType,
    properties: Vec<Properties>,
}

impl ComposedItem {
    fn new(src: &Source, path: &PathBuf, len: usize) -> RepubResult<Self> {
        let media_type = MediaType::try_from(path)?;
        let id = format!("{}{}", media_type.to_string(), len);

        Ok(Self {
            src: src.clone(),
            path: path.clone(),
            id,
            media_type,
            properties: Vec::new(),
        })
    }
}

pub mod media_type {
    use super::*;

    /// https://imagedrive.github.io/spec/epub30-publications.xhtml#tbl-core-media-types
    #[derive(Clone, PartialEq)]
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
                .ok_or(format_err!("{}",RepubError(format!("EPUB3は拡張子 {} に対応していません",s))))
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
                MediaType::Image(t) => t.to_string(),
                MediaType::Application(t) => t.to_string(),
                MediaType::Audio(t) => t.to_string(),
                MediaType::Text(t) => t.to_string(),
            }
        }
    }

    impl TryFrom<&PathBuf> for MediaType {
        type Error = failure::Error;

        fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
            let ext = value.extension().ok_or(
                format_err!("{}",RepubError(format!("Failed to unwrap Option<&OsStr> on {}:{}",line!(),column!())))
            )?.to_str().ok_or(
                format_err!("{}",RepubError(format!("Failed to unwrap Option<&str> on {}:{}",line!(),column!())))
            )?;

            Ok(MediaType::from_str(&ext)?)
        }
    }

    #[derive(Clone, PartialEq)]
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

    #[derive(Clone, PartialEq)]
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
                "xhtml" | "xht" | "html" | "htm" => Ok(ApplicationType::XHTML),
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

    #[derive(Clone, PartialEq)]
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
                AudioType::MPEG => "audio/mpeg",
                AudioType::MP4 => "audio/mp4",
            }.to_string()
        }
    }

    #[derive(Clone, PartialEq)]
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
                TextType::CSS => "text/css",
                TextType::JS => "text/javascript",
            }.to_string()
        }
    }
}

pub mod properties {
    /// https://imagedrive.github.io/spec/epub30-publications.xhtml#sec-item-property-values
    #[derive(Clone)]
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
                    println!("found ol tag");
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