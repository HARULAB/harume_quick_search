# HARUME Quick Search

AviUtl2 用のエフェクト検索プラグインです。 
AEで言うとFX Consoleと似た機能を持ちます

![Preview](preview.png)

![AviUtl2](https://img.shields.io/badge/AviUtl2-plugin-blue)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

## 開発について
本プラグインのソースコードおよび構成の作成にあたっては、生成AIを活用しています。

---

## 使い方

1. AviUtl2 の編集メニューから **`HARUME Quick Search \ Open Panel`** を選択します。またはショートカット設定から割り当てます
2. 検索ボックスに入力します。
3. クリックまたは **Enter** キーで、選択中のオブジェクト、タイムラインに追加されます。

---

## 動作確認済み環境

- AviUtl2 beta47
  - ※作者の環境でのみテストしています。他の環境での動作は保証されません。

---

## ライセンス

### 本プロジェクト
MIT License (c) 2026 HARULAB  
詳細は `LICENSE` ファイルをご覧ください。

### クレジット
本プロジェクトでは以下のライブラリを利用しています。
* [AviUtl2 SDK (aviutl2-rs)](https://github.com/sevenc-nanashi/aviutl2-rs)
* [egui / eframe](https://github.com/emilk/egui)
* [windows-rs](https://github.com/microsoft/windows-rs)
* [serde](https://github.com/serde-rs/serde)

---

免責事項
このプラグインを使用したことによって生じたすべての障害・不具合等に関しては、作成者およびその関係者は一切の責任を負いません。各自の責任においてご使用ください。
