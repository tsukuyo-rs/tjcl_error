# tjcl_error

[![CI](https://github.com/tsukuyo-rs/tjcl_error/actions/workflows/ci.yml/badge.svg)](https://github.com/tsukuyo-rs/tjcl_error/actions/workflows/ci.yml)
[![License: MIT-0](https://img.shields.io/badge/License-MIT--0-blue.svg)](LICENSE)
[![no_std](https://img.shields.io/badge/Rust-no__std-lightgrey.svg)](https://docs.rust-embedded.org/)

> 🌐 **English Guide**: An English overview is available [below](#english).

**車載・組み込み通信向け、ビットコンパクト・ゼロアロケーションのエラー管理パッケージひな型（`no_std` / Rust）**

---

## 概要

`tjcl_error` は、CAN 通信、シリアル通信（UART/SPI/I2C）、Flash / EEPROM ログ記録など、**通信帯域やストレージ領域が制約された組み込みシステム**のために設計されたエラー管理パッケージのひな型です。

外部ライブラリ（crates.io）として固定されたブラックボックスではなく、**「プロジェクトに取り込み、基板仕様に合わせて書き換えて使う白箱」**として設計されています。

```
[プログラム内]                          [通信・Flash記録時]
型安全な親子Enum                        固定長ビット列
ErrorKind::I2C(I2cSub::AddressNack)  <--->  0x1003 (16-bit / 2バイト)
```

---

## 解決する課題

### 1. 通信帯域への配慮（Bit Conservation）
CAN フレーム（ペイロード最大8バイト）や低帯域な通信パケットでは、エラー情報を大きな整数や可変長文字列として送ることが難しい場面があります。本ひな型は、親カテゴリ（Kind: 8bit）と詳細（Detail: 8bit）をパッキングし、**2 バイト（u16）** として扱えるよう設計しています（4bit + 4bit の 1 バイト運用への改変例も想定しています）。

### 2. 手動管理の煩雑さを Rust マクロで解消
C言語では、エラー番号の定義・ビットシフト・デコード用 `switch-case`・文字列テーブルをそれぞれ別々に管理する必要があり、追加・変更の際に対応漏れが起きやすい構造でした。本ひな型は、**Rust の標準マクロ（`macro_rules!`）により 1 箇所の定義から関連する実装をまとめて生成（Single Source of Truth）** する構成をとっています。

### 3. コンパイラによる網羅性チェック
新しいエラーを定義した際、対応する文字列の追加を忘れると、**Rust コンパイラが網羅性エラー（E0004）を出力してビルドを停止**します。表示の抜けがサイレントに混入しにくい設計になっています。

---

## 5つの特徴

1. **`#![no_std]` ＆ ゼロ動的確保（Zero Allocation）**
   ヒープメモリ（`alloc`）を使用せず、動的確保に伴うパニックやメモリ断片化のリスクを避けています。文字列は Flash メモリ上の `&'static str` を直接参照します。
2. **外部クレート依存ゼロ**
   `syn` や `quote` 等の手続き型マクロを使わず、標準の `macro_rules!` だけで完結しているため、ビルドへの影響を最小限に抑えられます。
3. **決定論的な動作**
   エンコードはビットシフトと論理和（`LSL`, `ORR`）のみで実行されるため、WCET（最悪実行時間）を把握しやすい構造です。すべて `const fn` であり、コンパイル時評価にも対応しています。
4. **徹底したカプセル化**
   内部モジュール構造は非公開（`mod`）に閉じ込められ、トップレベルの `pub use` のみで利用できる明示的な API 設計となっています。
5. **高いカスタマイズ性**
   `u16` だけでなく、ニブル分割による `u8`（4bit + 4bit）や、車載 DTC 向けの `u32` など、プロジェクトの要件に応じてマクロの数行を変更することでスケールできるよう設計しています。

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

define_error_detail!(
    /// I2C通信サブエラー詳細
    I2cSub {
        /// バスビジー
        BusBusy = 0x01,
        /// アドレス送信時のNACK応答
        AddressNack = 0x03,
        /// 通信タイムアウト
        Timeout = 0x05,
    }
);
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

本ソフトウェアは **[MIT-0 (MIT No Attribution License)](LICENSE)** のもとで公開されています。商用・非商用を問わず、どなたでも自由にご利用・改変・組み込みいただけます。

* **表示義務なし（No Attribution Required）**:
  クレジット表記や著作権表示、ライセンス条文の保持義務は **一切不要** です。ご自身のファームウェアや製品コード内にコピー＆ペーストし、基板仕様に合わせて自由に書き換えてご利用いただけます（企業の法務審査や製品マニュアルへのライセンス表記負担も発生しません）。
* **オナーシステム（クレジット表記のお願い）**:
  法的な義務付けはいたしませんが、もし本ひな型が業務の効率化や製品開発のお役に立ちましたら、リポジトリへのStarや、コードヘッダー・ドキュメントの片隅などにクレジット（`Tsukuyomi Japan Code Lab` / `TJCL`）を残していただければ大変励みになります。

---

## English

### Overview

`tjcl_error` is a bit-compact, zero-allocation error management package template in Rust (`no_std`), designed for **bandwidth- and storage-constrained embedded systems** such as CAN bus, serial communications (UART/SPI/I2C), and Flash/EEPROM logging.

Rather than being a rigid external dependency (crates.io black-box), it is designed as a **"white-box template to be integrated directly into your project and adapted to your hardware specifications."**

```
[In-Program]                           [Communication / Flash Logging]
Type-safe hierarchical enums           Fixed-width bit sequence
ErrorKind::I2C(I2cSub::AddressNack)  <--->  0x1003 (16-bit / 2 bytes)
```

---

### Problems Solved

#### 1. Bit Conservation for Bandwidth Constraints
In CAN frames (max payload of 8 bytes) and low-bandwidth communication packets, sending error details as large integers or variable-length strings is often impractical. This template packs a parent category (Kind: 8-bit) and detail (Detail: 8-bit) into **2 bytes (`u16`)** (with adaptability to a 1-byte `u8` / 4-bit + 4-bit layout).

#### 2. Eliminating Manual Maintenance Overhead with Rust Macros
In traditional C firmware, error codes, bit-shift logic, decode `switch-case` statements, and string lookup tables had to be maintained separately—frequently causing desynchronization bugs when adding or modifying codes. This template uses **standard Rust declarative macros (`macro_rules!`) as a Single Source of Truth**, generating all associated implementations from a single definition.

#### 3. Compiler-Enforced Exhaustiveness Checking
When defining a new error variant, forgetting to add its corresponding string representation causes the **Rust compiler to halt the build with an exhaustiveness check error (E0004)**. This prevents missing string mappings from silently leaking into production firmware.

---

### 5 Key Features

1. **`#![no_std]` & Zero Dynamic Allocation**
   Zero heap allocation (`alloc`). Eliminates the risk of panics, heap exhaustion, or memory fragmentation. All string representations reference `&'static str` located directly in Flash memory.
2. **Zero External Crate Dependencies**
   Built exclusively with standard `macro_rules!`—no procedural macros like `syn` or `quote`—minimizing build times and dependency trees.
3. **Deterministic Execution**
   Encoding is performed solely through bit shifts and bitwise OR (`LSL`, `ORR`), making WCET (Worst-Case Execution Time) highly predictable. All encoding and decoding routines are `const fn`, fully supporting compile-time evaluation.
4. **Thorough Encapsulation**
   Internal submodule structures are kept strictly private (`mod`), exposing only clean, intentional APIs via top-level `pub use`.
5. **High Customizability**
   Easily tailored to specific system requirements—from `u16` to nibble-split `u8` (4-bit + 4-bit) or 32-bit automotive DTC (Diagnostic Trouble Code) formats—by adjusting just a few lines in the macros.

---

### Directory Structure

```
tjcl_error/
├── Cargo.toml            # Workspace manifest
├── common_error/         # [Core] Error type definitions & u16 packing macros
│   └── src/
│       ├── lib.rs        # Public API (clean pub use encapsulation)
│       └── error/
│           ├── macros.rs # define_error_kind! / define_error_detail!
│           ├── kind.rs   # Parent enum definitions
│           ├── i2c.rs    # Child enum definitions (I2C example)
│           └── system.rs # Child enum definitions (System example)
├── format/               # [Formatting] Flash static string conversion traits
│   └── src/
│       ├── lib.rs
│       └── format.rs     # ErrorFormat trait implementation
└── tests/                # [Tests] Isolated test crate avoiding production bloat
    └── src/
        └── lib.rs        # Round-trip conversion, undefined code, and string formatting tests
```

---

### Quick Start

#### 1. Defining Child Errors (e.g., `i2c.rs`)
Define peripheral-specific error details using the `define_error_detail!` macro:

```rust
use crate::define_error_detail;

define_error_detail!(
    /// I2C communication sub-error details
    I2cSub {
        /// Bus busy
        BusBusy = 0x01,
        /// NACK response on address transmission
        AddressNack = 0x03,
        /// Communication timeout
        Timeout = 0x05,
    }
);
```

#### 2. Registering with Parent Error (`kind.rs`)
Explicitly import the child error at the top of the file and bind it to a parent category ID (8-bit):

```rust
use crate::define_error_kind;
use crate::error::i2c::I2cSub;

define_error_kind! {
    /// I2C peripheral communication error (Parent ID: 0x10)
    I2C = 0x10 => I2cSub,
}
```

#### 3. Usage Example

```rust
use tjcl_error_common::{ErrorKind, I2cSub};
use tjcl_error_format::ErrorFormat;

let err = ErrorKind::I2C(I2cSub::AddressNack);

// 1. Pack into a 16-bit integer for CAN/Flash transmission (0x10 << 8 | 0x03 = 0x1003)
let raw_code: u16 = err.to_u16();
assert_eq!(raw_code, 0x1003);

// 2. Unpack from received raw integer (safely returns None for undefined codes)
let restored = ErrorKind::from_u16(raw_code);
assert_eq!(restored, Some(err));

// 3. String representation (zero-allocation / static strings in Flash)
let kind_name   = err.kind_str();   // "I2C"
let detail_name = err.detail_str(); // "AddressNack"
let full_name   = err.full_str();   // "I2C::AddressNack"
```

---

### Running Tests

Run all workspace tests from the repository root:

```bash
cargo test
```

---

### License & Attribution

This software is released under the **[MIT-0 (MIT No Attribution License)](LICENSE)**. You are free to use, modify, and integrate it into any commercial or non-commercial product.

* **No Attribution Required**:
  Retaining copyright notices, credits, or the license text is **completely optional**. You may copy and paste the code directly into your firmware or product codebase and modify it to match your board specifications (incurring zero legal audit or product manual documentation burden).
* **Honor System (Attribution Appreciated)**:
  While not legally required, if this template helps streamline your workflow or product development, a repository Star or a small credit note (`Tsukuyomi Japan Code Lab` / `TJCL`) in code headers or documentation would be greatly appreciated.

---

Copyright (c) 2026 Tsukuyomi Japan Code Lab
