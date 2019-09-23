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

### contents *experimental*
このフィールドは省略可能です. 省略しない場合、オブジェクトの配列を指定します. 
オブジェクトは以下のような形です. 

```json
{
  "src": "relative/path/from/repub_config.json/to/content_file",
  "properties": ["nav"],
  "styles": [
    "relative/path/from/repub_config.json/to/css_file0.css",
    "relative/path/from/repub_config.json/to/css_file1.css"
  ]
}
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
    
***このフィールドが省略されていない場合, 配列に入っているオブジェクトに対応するコンテンツのみが`.epub`にパッケージされます. この挙動は[v0.4.1](https://github.com/KaiseiYokoyama/repub/issues/39)で変更する予定です***.
*このフィールドは省略可能です*.