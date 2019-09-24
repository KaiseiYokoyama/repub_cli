# Usage
ディレクトリを変換する時のみ, 本機能は有効です. 

`repub`コマンドの実行時, `--config`オプションを追加することで, 変換に利用した諸設定を保存することができます.


```bash
repub config --config
```

このとき, configディレクトリには`repub_config.json`が生成されます. 

```json
{
  "target": "config",
  "writing_mode": "htb",
  "title": "config",
  "creator": "me",
  "language": "ja",
  "book_id": "SBDGInK1nP9YHIg6nylpaK25IKFtFw",
  "toc_depth": 2,
  "verbose": false,
  "save": false,
  "config": true,
  "cover_image": null,
  "ignores": [
    "repub_config.json",
    ".DS_Store"
  ],
  "contents": null
}
```

`repub`コマンドを実行すると, 対象ディレクトリに`repub_config.json`がある場合は, まずそちらの設定を読み取ります. 
コマンドに引数がある場合, パラメータは上書きされ, オプションは反転します.

## keys
コマンド引数と同じものは省略.

### ignores
ここにパスを指定されたファイルは, 変換時に無視されます. パスは`repub_config.json`からの相対パスです. 
*このフィールドは省略可能です*. 

### sequence
コンテンツの並び順を指定します. 
```json
[
  "sample1.md",
  "sample2.md",
  "sample3.md"
]
```

このフィールドが存在する場合, 並び順を指定されなかったコンテンツのファイルは収録されません. 
*このフィールドは省略可能です*. 

### content_configures
コンテンツに適用する`.css`ファイルや property を指定します. 
```json
[
  {
    "src": "configured.md",
    "properties": ["nav"],
    "styles": ["special.css", "more_special.css"]
  }
]
```

 - src
    - コンテンツのソースのパスです
 - properties
    - コンテンツにプロパティを指定します. 
    - 指定できるプロパティについては, https://imagedrive.github.io/spec/epub30-publications.xhtml#sec-item-property-values を参考にしてください. 
    - *このフィールドは省略可能です*.
 - styles
    - コンテンツに適用する`.css`ファイルを指定します
    - 配列の要素は`.css`ファイルへの`repub_config.json`からの相対パスです. 
    - *このプロパティは省略可能です*. 

*このフィールドは省略可能です*. 