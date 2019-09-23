# repub [![Build Status](https://travis-ci.com/KaiseiYokoyama/repub.svg?branch=master)](https://travis-ci.com/KaiseiYokoyama/repub) [![dependency status](https://deps.rs/repo/github/KaiseiYokoyama/repub/status.svg)](https://deps.rs/repo/github/KaiseiYokoyama/repub) ![GitHub](https://img.shields.io/github/license/KaiseiYokoyama/repub) [![Crates.io](https://img.shields.io/crates/v/repub.svg?maxAge=2592000)](https://crates.io/crates/repub)
![logo_2_2](https://user-images.githubusercontent.com/8509057/65156816-addf6c80-da6a-11e9-8d16-74ef4a47a90e.png)

markdown 文書を 電子書籍 (epub3) に変換します. 
![cover_image](https://user-images.githubusercontent.com/8509057/64936464-0ce98980-d891-11e9-97f9-72925653c4ba.png)

## Install
まず, Rust 開発環境を整えてください. 
```bash
curl https://sh.rustup.rs -sSf | sh
```

***repubをインストール***

```bash
cargo install repub
```

### Update
```bash
cargo install --force repub
```

### Uninstall
```bash
cargo uninstall repub
```

## Usage
[usage.md](examples/usage/usage.md)を御覧ください. 

```
$ repub --help
repub 0.4.0
Kaisei Yokoyama <yokoyama.kaisei.sm@alumni.tsukuba.ac.jp>
A tool to convert markdown documents to epub.

USAGE:
    repub [FLAGS] [OPTIONS] <input>

FLAGS:
        --config     設定ファイルを保存
    -h, --help       Prints help information
        --save       一時ファイルを消去しない
    -V, --version    Prints version information
        --verbose    ログを表示

OPTIONS:
    -i, --bookid <book_id>             Book ID
        --cover-image <cover_image>    表紙 / Cover image
    -c, --creator <creator>            作者、編集者、翻訳者など
    -l, --language <language>          言語
    -t, --title <title>                タイトル
        --toc-depth <toc_depth>        目次に表示するHeaderの最低レベル(1~5)
        --mode <writing_mode>          縦書き / 横書き [default: htb]  [possible values: htb, vrl, vlr]

ARGS:
    <input>    変換するマークダウンファイル OR 変換するマークダウン文書(複数可)の入ったディレクトリ OR 設定ファイル
```

# History
https://github.com/KaiseiYokoyama/repub/releases