# tjcl_error

[![CI](https://github.com/ユーザー名/tjcl_error/actions/workflows/ci.yml/badge.svg)](https://github.com/ユーザー名/tjcl_error/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![no_std](https://img.shields.io/badge/Rust-no__std-lightgrey.svg)](https://docs.rust-embedded.org/)

**車載・組み込み通信向け、ビット節約・ゼロアロケーションのエラー管理パッケージひな型（`no_std` / Rust）**

---

## 概要

`tjcl_error` は、CAN 通信、シリアル通信（UART/SPI/I2C）、Flash / EEPROM ログ記録など、**通信帯域やストレージ領域が極限まで制約された組み込みシステム**のために設計されたエラー管理パッケージのひな型（テンプレート）です。

外部ライブラリ（crates.io）として固定されたブラックボックスではなく、**「プロジェクト内にコピー＆ペーストし、基板のハードウェア仕様に合わせて自由に書き換えて使う白箱」**として構成されています。

```
[プログラム内]                          [通信・Flash記録時]
型安全な親子Enum                        極小ビット列（固定長）
ErrorKind::I2C(I2cSub::AddressNack)  <--->  0x1003 (16-bit / 2バイト)
```

---

## 解決する課題

### 1. 極限のビット節約（Bit Conservation）
CAN フレーム（ペイロード最大8バイト）や低帯域な無線・通信パケットにおいて、32bit 以上の巨大な整数や可変長文字列でエラーを送ることはできません。本ひな型は、親カテゴリ（Kind: 8bit）と詳細（Detail: 8bit）を綺麗にパッキングし、**わずか 2 バイト（u16）** に圧縮します（4bit + 4bit の 1 バイト運用への改変も容易です）。

### 2. C言語時代の `#define` 地獄・ビットずれの撲滅
C言語では、エラー番号の定義・ビットシフト・デコード用 `switch-case`・文字列テーブルを手動管理していたため、ビットのズレやバリアント追加漏れによるデバッグ工数が膨大でした。本ひな型は、**Rust の標準マクロ（`macro_rules!`）により 1 箇所の定義から全機能を一括自動生成（Single Source of Truth）** します。

### 3. コンパイラによる「追加漏れ」の 100% 検知
新しいエラーを定義した際、表示用文字列の追加を忘れると、**Rust コンパイラが網羅性エラー（E0004）を出してビルドを停止**します。サイレントな表示抜けや未定義バグが原理的に発生しません。

---

## 5つの特徴

1. **完全な `#![no_std]` ＆ ゼロ動的確保（Zero Allocation）**
   ヒープメモリ（`alloc`）を 1 バイトも使用せず、動的確保によるパニックやメモリ断片化の恐れがありません。文字列は Flash メモリ上の `&'static str` を直接参照します。
2. **外部クレート依存ゼロ**
   `syn` や `quote` 等の重厚な手続き型マクロを使わず、標準の `macro_rules!` だけで完結しているため、ビルドが一瞬で完了します。
3. **決定論的かつ超高速（数クロックサイクル）**
   エンコードはビットシフトと論理和（`LSL`, `ORR`）のみで実行され、WCET（最悪実行時間）が完全に予測可能です。すべて `const fn` であり、コンパイル時確定が可能です。
4. **徹底したカプセル化**
   内部モジュール構造は非公開（`mod`）に閉じ込められ、トップレベルの `pub use` のみで利用できるクリーンな API 設計となっています。
5. **自由に弄れる高い改変自由度**
   `u16` だけでなく、ニブル分割による `u8`（4bit + 4bit）や、上位のみの `u8`、車載 DTC 向けの `u32` など、プロジェクトの要件に応じてマクロの数行を書き換えるだけで自由自在にスケールします。

---

## ディレクトリ構成

```
tjcl_error/
├── Cargo.toml            # ワークスペース定義
├── common_error/         # 【コア】エラーの型定義・u16パッキングマクロ
│   └── src/
│       ├── lib.rs        # 公開API（pub use カプセル化）
│       └── error/
│           ├── macros.rs # define_error_kind! / define_error_detail!
│           ├── kind.rs   # 親Enum定義
│           ├── i2c.rs    # 子Enum定義（I2Cの例）
│           └── system.rs # 子Enum定義（Systemの例）
├── format/               # 【文字列表現】Flash静的文字列変換トレイト
│   └── src/
│       ├── lib.rs
│       └── format.rs     # ErrorFormat トレイト実装
└── tests/                # 【テスト】本番コードを汚さない隔離テスト子クレート
    └── src/
        └── lib.rs        # 往復変換・未定義コード・文字列化の検証テスト
```

---

## クイックスタート

### 1. 子エラーの定義（例: `i2c.rs`）
`define_error_detail!` マクロを用いて、ペリフェラル固有のエラー詳細を定義します：

```rust
use crate::define_error_detail;

/// I2C通信サブエラー詳細
define_error_detail!(I2cSub {
    /// バスビジー
    BusBusy = 0x01,
    /// アドレス送信時のNACK応答
    AddressNack = 0x03,
    /// 通信タイムアウト
    Timeout = 0x05,
});
```

### 2. 親エラーへの登録（`kind.rs`）
ファイル先頭で子エラーを明示的に `use` し、親ID（8bit）と結びつけます：

```rust
use crate::define_error_kind;
use crate::error::i2c::I2cSub;

define_error_kind! {
    /// I2Cペリフェラル通信異常 (親ID: 0x10)
    I2C = 0x10 => I2cSub,
}
```

### 3. 利用コード例

```rust
use tjcl_error_common::{ErrorKind, I2cSub};
use tjcl_error_format::ErrorFormat;

let err = ErrorKind::I2C(I2cSub::AddressNack);

// ① CAN/Flash送信用の 16-bit 整数にパック (0x10 << 8 | 0x03 = 0x1003)
let raw_code: u16 = err.to_u16();
assert_eq!(raw_code, 0x1003);

// ② 受信した整数データからの復元（未定義コードは安全に None）
let restored = ErrorKind::from_u16(raw_code);
assert_eq!(restored, Some(err));

// ③ 文字列の取得（ゼロアロケーション / Flash上の静的文字列）
let kind_name   = err.kind_str();   // "I2C"
let detail_name = err.detail_str(); // "AddressNack"
let full_name   = err.full_str();   // "I2C::AddressNack"
```

---

## テストの実行

ルートディレクトリからワークスペース全体を一括テストできます：

```bash
cargo test
```

---

## ライセンスとクレジットについて (License & Attribution)

本ソフトウェアは **[MIT License](LICENSE)** のもとで公開されています。商用・非商用を問わず、どなたでも自由にご利用・改変いただけます。

* **個人でのご利用**:
  クレジット表記や著作権表示の保持は **不要** です。ご自身のプロジェクトに取り込み、自由に改造してお使いください。
* **法人・企業でのご利用**:
  本ひな型の利用にあたり、法的な強制や厳格な義務付けはいたしません。**各企業・開発者の倫理（オナーシステム）** にお任せいたします。
  もし本ひな型が業務の効率化や製品開発のお役に立ちましたら、社内リポジトリのコードヘッダーや、製品ドキュメントの片隅などにクレジット（`Tsukuyomi Code Lab`）を残していただければ大変励みになります。

---

Copyright (c) 2026 Tsukuyomi Code Lab
