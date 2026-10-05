# 詰めどうぶつしょうぎの自動生成（好みの反映機能つき）

## ユーザ向け

[デモ](https://kaorahi.github.io/playground2026/tsume_dobutsu_shogi/tsume_dobutsu_shogi.html)

## 開発者向け

内容

* `tsume_dobutsu_shogi.html` … 本体
* `tsume_dobutsu_shogi_data/` … 事前解析データ (なくても動作するが, あると効率的)
* `xz-decompress-0.2.3.min.js` … 事前解析データの展開に使用
* `retrograde-source/` … 事前解析データの生成に用いたコード

補足

* 「事前解析データ」は n 手詰局面を事前に全列挙したもの (normal = 詰将棋モード, fairy = 自由手モード). 既存の解析 ([1](https://www.tanaka.ecc.u-tokyo.ac.jp/ktanaka/dobutsushogi/), [2](https://github.com/mame/dobutsu-shogi-master), [3](https://github.com/kaorahi/dobutsu-shogi-master)) とは手数の数え方が違うことに注意. (キャッチは指さず詰みまでで止めるが, トライは実際に指す. 非対称だけれど, 実際遊んだ感覚ではこうしたくなった)
* 「事前解析データ」生成用コードを Google Colaboratory で実行するには, まず `!apt install cargo` をしておくこと. (2026-09-24 現在. `!make normal-all`と`!make fairy-all`が各1〜1.5時間程度)
* [xz-decompress](https://www.npmjs.com/package/xz-decompress) の入手は `npm install --save xz-decompress` や `npm pack xz-decompress@0.2.3`

リンク

* [GitHub](https://github.com/kaorahi/playground2026/)
