use clap::{App, Arg};

pub fn app<'a,'b>() -> App<'a,'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        // .mdファイルorフォルダ
        .arg(Arg::from_usage("<input> '変換するマークダウンファイル OR 変換するマークダウン文書(複数可)の入ったディレクトリ OR 設定ファイル'")
            .validator(validators::md_validator))
        // 一時ファイルを消さない
        .arg(Arg::with_name("save_tmp_files")
            .help("一時ファイルを消去しない")
            .long("save"))
        // 設定ファイルを出力
        .arg(Arg::with_name("save_config_files")
            .help("設定ファイルを保存")
            .long("config"))
        // タイトル
        .arg(Arg::with_name("title")
            .help("タイトルを設定")
            .short("t")
            .long("title")
            .takes_value(true))
        // 著者
        .arg(Arg::with_name("creator")
            .help("作者、編集者、翻訳者など")
            .short("c")
            .long("creator")
            .takes_value(true))
        // 言語
        .arg(Arg::with_name("language")
            .help("言語")
            .short("l")
            .long("language")
            .takes_value(true))
        // id
        .arg(Arg::with_name("book_id")
            .help("Book ID")
            .short("id")
            .long("bookid")
            .takes_value(true))
        // 縦書き
        .arg(Arg::with_name("writing_mode")
            .help("縦書き / 横書き")
            .long("mode")
            .possible_values(&["htb", "vrl", "vlr"])
            .default_value("htb"))
        // tocに載せるヘッダーのレベル
        .arg(Arg::with_name("toc_level")
            .help("目次に表示するHeaderの最低レベル(1~5)")
            .short("h")
            .takes_value(true))
}