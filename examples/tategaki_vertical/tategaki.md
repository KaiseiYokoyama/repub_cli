# repub 縦書き
repub を使えば, markdown 文書を **縦書き** の電子書籍に変換することもできます.
以下のコマンドを examples ディレクトリで実行してください.

```bash
repub tategaki_vertical  --mode vrl
```

examples ディレクトリに`.epub`ファイルが生成されます.

`vrl`とは `vartical-rl`の略です. *縦書き* で、なおかつ行は *右から左* に進みます.
`--mode`オプションには, 他に

- `vlr`: *縦書き* で, なおかつ行は *右から左* に進みます
- `htb`: *横書き* です **初期値**

を指定することができます.

**`--mode`オプションで指定しているのは, `page-progressing-direction`つまりページ送りの方向です. ページの中身の文字を縦書きにするには, 別途`css`を用意してスタイルをつけてやる必要があります.**
最も簡易な指定は, [tategaki.css](tategaki.css)を参考にすると良いでしょう. 細かいことにこだわらないのであれば, 自分の`.css`ファイルに以下のコードをコピーアンドペーストするか, [tategaki.css](tategaki.css)を直接ディレクトリにコピーして変換すると縦書きの電子書籍ができます.

```css
html {
    writing-mode: vertical-rl;
}
```
