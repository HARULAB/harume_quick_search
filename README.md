# HARUME Quick Search

AviUtl2 用のエフェクト検索プラグインです。  
After Effects の「FX Console」のように、素早くエフェクトを検索し追加できます。

![Preview](preview.png)

![AviUtl2](https://img.shields.io/badge/AviUtl2-plugin-blue)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

## 特徴

* **使用頻度に応じた並び替え**
  よく使うエフェクトは上位に表示されます。
* **カテゴリタブ**
  「すべて」「フィルタ」「オブジェクト」「シーン」の4タブで素早く絞り込めます。
* **キーボード操作**
  上下キーで選択、Enterで追加、Escで閉じる、左右キーでタブ切り替えが可能です。

---

## 使い方

1. 編集メニューから **`HARUME Quick Search \ Open Panel`** を選択（ショートカットキーへの割り当てを推奨）。
2. 検索ボックスに名前を入力します。
3. クリックまたは **Enter** キーで、選択中のオブジェクトやタイムラインに追加されます。

---

## 動作確認環境

* AviUtl2 beta47
※開発者の環境でのみテストしています。すべての環境での動作を保証するものではありません。

---

## 開発について

本プラグインのコード作成にあたっては、生成AIを活用した開発支援を受けています。

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

## 免責事項

本プラグインの使用によって生じたすべての障害・損害について、制作者は一切の責任を負いません。各自の責任においてご使用ください。
