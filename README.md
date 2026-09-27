# tjcl_error

[![CI](https://github.com/tsukuyo-rs/tjcl_error/actions/workflows/ci.yml/badge.svg)](https://github.com/tsukuyo-rs/tjcl_error/actions/workflows/ci.yml)
[![License: MIT-0](https://img.shields.io/badge/License-MIT--0-blue.svg)](LICENSE)
[![no_std](https://img.shields.io/badge/Rust-no__std-lightgrey.svg)](https://docs.rust-embedded.org/)

> 🌐 **English Guide**: An English overview is available [below](#english).

**車載・組み込み通信向け、ビットコンパクト・ゼロアロケーションのエラー管理パッケージひな型（`no_std` / Rust）**

---

## 概要

`tjcl_error` は、Classic CAN 通信、シリアル通信（UART/SPI/I2C）、Flash / EEPROM ログ記録など、**通信帯域やストレージ領域が制約された組み込みシステム**のためのエラー管理テンプレートです。デバッグ出力やキャラクタLCD/OLED等の表示器を備えたシステムへの適用も考慮しています。

外部の固定化されたクレートに依存し続けるのではなく、**「プロジェクトに取り込み、ハードウェア仕様や通信規約に合わせて書き換えて使う白箱」**として設計されています。

```text
[プログラム内]                          [通信・Flash記録時]
型安全な親子Enum                        固定長ビット列
ErrorKind::I2C(I2cSub::AddressNack)  <--->  0x1003 (16-bit / 2バイト)
```

---

## 解決する課題

### 1. 通信帯域・ストレージへの配慮（Bit Conservation）
Classic CAN フレーム（ペイロード最大 8 バイト）や低帯域シリアル通信では、エラー情報を大きなデータや可変長文字列のまま送信することは非効率です。本ひな型では、親カテゴリ（Kind: 8bit）と詳細（Detail: 8bit）を 1 つの **16ビット整数（`u16`）** にパッキングして扱います。

### 2. マクロによる型定義と変換ロジックの一元化
C言語等で発生しやすい「エラー番号定義」「ビットシフト処理」「復元用 `switch-case`」の手動二重管理を解消します。標準マクロ（`macro_rules!`）により、Enum定義から相互変換関数（`to_u16` / `from_u16`）を一括生成します。

### 3. 用途に応じたクレート分離（制御系と表示・ログ系の両立）
文字列変換ロジックを別クレート（`format`）に分離しています。文字列を必要としない制御系タスクなどでは `common_error` のみを取り込み、文字列機能を取り込まない構成にできます。一方、表示器やログ機能が必要なノードでは `format` を追加導入することで、同じエラー定義を共有できます。

### 4. コンパイラによる網羅性チェック（文字列機能利用時）
`format` クレートを導入している構成では、新しいエラーバリアントを追加した際に文字列化 `match` 式の更新を忘れると、**Rust コンパイラが網羅性エラー（E0004）を出力**してビルドを停止します。文字列対応の抜け漏れにコンパイル段階で気付くことができます。

---

## 主な特徴

1. **`#![no_std]` ＆ 動的メモリ確保なし（Zero Allocation）**  
   本パッケージはヒープ確保（`alloc`）を行わず、ヒープ枯渇や断片化の原因になりません。文字列機能も静的領域の文字列スライス（`&'static str`）を直接返します。
2. **外部依存の排除**  
   `std` やサードパーティ製クレートに依存しません（`syn` や `quote` などの手続き型マクロも不使用）。
3. **単純で予測しやすいビット演算**  
   パッキング処理はシフト演算とビットORのみで行われ、`const fn` に対応しています。
4. **責務を分離したクレート構成**  
   コアのエラー定義（`common_error`）と文字列表現（`format`）が分離されており、要件に合わせて必要な機能のみを選択できます。
5. **ビットレイアウトの改変可能性**  
   提供している `u16`（8bit + 8bit）実装をベースに、要件に応じて 4bit + 4bit（`u8`）や車載 DTC 形式（24/32bit）への改変ひな型として活用できます。

---

## ディレクトリ構成

```text
tjcl_error/
├── Cargo.toml            # ワークスペース定義
├── common_error/         # 【コア】エラーの型定義・u16パッキングマクロ（依存クレートなし）
│   └── src/
│       ├── lib.rs        # 公開API
│       └── error/
│           ├── macros.rs # define_error_kind! / define_error_detail!
│           ├── kind.rs   # 親Enum定義
│           ├── i2c.rs    # 子Enum定義（I2Cの例）
│           └── system.rs # 子Enum定義（Systemの例）
├── format/               # 【文字列表現】静的文字列変換トレイト（common_errorに依存）
│   └── src/
│       ├── lib.rs
│       └── format.rs     # ErrorFormat トレイト実装
└── tests/                # 【テスト】検証用テスト
    └── src/
        └── lib.rs        # 往復変換・未定義コード・文字列化の検証テスト
```

---

## クイックスタート

### 1. エラーの定義（`common_error` 側）

子エラー（詳細）と親カテゴリをマクロで定義します：

```rust
// 子エラー（例: i2c.rs）
use crate::define_error_detail;

define_error_detail!(
    I2cSub {
        BusBusy = 0x01,
        AddressNack = 0x03,
        Timeout = 0x05,
    }
);

// 親カテゴリ（例: kind.rs）
use crate::define_error_kind;
use crate::error::i2c::I2cSub;

define_error_kind! {
    I2C = 0x10 => I2cSub,
}
```

### 2. 利用パターンA: 文字列不要の最小構成（制御系マイコンなど）

`Cargo.toml` に `tjcl_error_common` のみを追加します。`format` を依存に加えないことで、文字列機能を取り込まない構成にできます。

```rust
use tjcl_error_common::{ErrorKind, I2cSub};

let err = ErrorKind::I2C(I2cSub::AddressNack);

// ① 16-bit 整数へのパッキング (0x10 << 8 | 0x03 = 0x1003)
let raw_code: u16 = err.to_u16();
assert_eq!(raw_code, 0x1003);

// ② 受信した整数データからの復元（未定義コードは None）
let restored = ErrorKind::from_u16(raw_code);
assert_eq!(restored, Some(err));
```

### 3. 利用パターンB: 文字列表現の利用（デバッグ・表示器向け）

文字列機能が必要な場合は、`tjcl_error_common` に加えて `tjcl_error_format` も依存に追加します。

```rust
use tjcl_error_common::{ErrorKind, I2cSub};
use tjcl_error_format::ErrorFormat;

let err = ErrorKind::I2C(I2cSub::AddressNack);

// &'static str を直接取得（ヒープ確保なし）
let kind_name: &'static str = err.kind_str();     // "I2C"
let detail_name: &'static str = err.detail_str(); // "AddressNack"
let full_name: &'static str = err.full_str();     // "I2C::AddressNack"

// 取得した文字列は、バッファコピー不要でそのまま表示APIやロガーに渡せます
// 例: display.draw_text(full_name);
// 例: defmt::info!("{=str}", full_name);
```

---

## カスタマイズ（ビットレイアウトの改変例）

本ひな型は親 8bit / 子 8bit の 16bit 設計です。4bit + 4bit（8bit）や車載 DTC（24/32bit）へ変更する場合は、`macros.rs` 内の型（`u8`/`u16`）、シフト幅、マスク処理を対象の要件に合わせて調整してください。

---

## ライセンスについて (License)

本ソフトウェアは **[MIT-0 (MIT No Attribution License)](LICENSE)** のもとで公開されています。

* **ライセンス概要**:  
  MIT-0 は、利用者に著作権表示やライセンス条文の保持（帰属表示）を求めないライセンスです。商用・非商用を問わず、自由に改変・組み込みいただけます。
* **クレジット表記について（任意）**:  
  必須ではありませんが、もし本ひな型がお役に立ちましたら、リポジトリへのStarやクレジット（`Tsukuyomi Japan Code Lab`）の任意記載をいただければ幸いです。

---

<!-- リポジトリの原典著作権表記 -->
Copyright (c) 2026 Tsukuyomi Japan Code Lab

---

## English

### Overview

`tjcl_error` is a bit-compact, zero-allocation error management package template in Rust (`no_std`), designed for **bandwidth- and storage-constrained embedded systems** such as Classic CAN, serial interfaces (UART/SPI/I2C), and Flash/EEPROM logging. It is also suitable for nodes equipped with displays (LCD/OLED) or debug logging channels.

Rather than acting as a rigid, black-box external dependency, it is designed as a **"white-box template to be integrated directly into your project and adapted to your hardware specifications and communication protocols."**

```text
[In-Program]                           [Communication / Flash Logging]
Type-safe hierarchical enums           Fixed-width bit sequence
ErrorKind::I2C(I2cSub::AddressNack)  <--->  0x1003 (16-bit / 2 bytes)
```

---

### Problems Solved

#### 1. Bit Conservation for Bandwidth Constraints
In Classic CAN frames (max payload of 8 bytes) and low-bandwidth serial links, transmitting error details as wide integers or variable-length strings is impractical. This template packs a parent category (Kind: 8-bit) and detail (Detail: 8-bit) into **2 bytes (`u16`)**.

#### 2. Macro-Driven Single Definition for Types and Conversions
Eliminates manual synchronization between error constants, bit-shifting routines, and decoding `switch-case` tables common in C firmware. Standard declarative macros (`macro_rules!`) generate enum definitions and bidirectional conversions (`to_u16` / `from_u16`) consistently.

#### 3. Crate Separation Based on System Requirements
String formatting logic is separated into an independent crate (`format`). Systems that do not require text output can depend solely on `common_error`, excluding string features from their build. Nodes requiring displays or debug logs can add `format` while sharing the exact same error types.

#### 4. Compiler-Enforced Exhaustiveness (When Using Strings)
When the `format` crate is integrated, failing to add a string mapping for a newly introduced error variant causes the **Rust compiler to halt the build with an exhaustiveness check error (E0004)**, preventing unmapped errors from going unnoticed.

---

### Key Features

1. **`#![no_std]` & Zero Heap Allocation**  
   This package performs no dynamic memory allocation (`alloc`) and will not cause heap exhaustion or fragmentation. The formatting crate directly returns static string slices (`&'static str`).
2. **No Dependency on `std` or Third-Party Crates**  
   No dependency on `std` or third-party crates (free of procedural macro overhead like `syn` or `quote`).
3. **Simple, Predictable Bitwise Operations**  
   Encoding relies purely on shifts and bitwise OR. All encoding and decoding routines are `const fn`.
4. **Decoupled Crate Architecture**  
   Core error definitions (`common_error`) and string formatting (`format`) are isolated, allowing users to include string features only when necessary.
5. **Adaptable Bit Layout**  
   The provided 16-bit (`u16`, 8+8 bits) implementation serves as a starting point that can be modified to 8-bit (`u8`, 4+4 bits) or 24/32-bit automotive DTC structures.

---

### Directory Structure

```text
tjcl_error/
├── Cargo.toml            # Workspace manifest
├── common_error/         # [Core] Error type definitions & u16 packing macros (no external dependencies)
│   └── src/
│       ├── lib.rs        # Public API
│       └── error/
│           ├── macros.rs # define_error_kind! / define_error_detail!
│           ├── kind.rs   # Parent enum definitions
│           ├── i2c.rs    # Child enum definitions (I2C example)
│           └── system.rs # Child enum definitions (System example)
├── format/               # [Formatting] Static string conversion traits (depends on common_error)
│   └── src/
│       ├── lib.rs
│       └── format.rs     # ErrorFormat trait implementation
└── tests/                # [Tests] Verification test suite
    └── src/
        └── lib.rs        # Round-trip conversion, undefined code, and string formatting tests
```

---

### Quick Start

#### 1. Defining Errors (in `common_error`)

Define child details and parent categories using macros:

```rust
// Child error (e.g., i2c.rs)
use crate::define_error_detail;

define_error_detail!(
    I2cSub {
        BusBusy = 0x01,
        AddressNack = 0x03,
        Timeout = 0x05,
    }
);

// Parent category (e.g., kind.rs)
use crate::define_error_kind;
use crate::error::i2c::I2cSub;

define_error_kind! {
    I2C = 0x10 => I2cSub,
}
```

#### 2. Usage Pattern A: Minimal Configuration without Strings (Control Systems)

Add only `tjcl_error_common` to your `Cargo.toml`. By omitting `format` from your dependencies, string functionality is excluded entirely.

```rust
use tjcl_error_common::{ErrorKind, I2cSub};

let err = ErrorKind::I2C(I2cSub::AddressNack);

// 1. Pack into 16-bit integer (0x10 << 8 | 0x03 = 0x1003)
let raw_code: u16 = err.to_u16();
assert_eq!(raw_code, 0x1003);

// 2. Unpack from raw integer (returns None for unmapped codes)
let restored = ErrorKind::from_u16(raw_code);
assert_eq!(restored, Some(err));
```

#### 3. Usage Pattern B: Using String Formatting (Displays & Debug Logs)

When human-readable strings are needed, add both `tjcl_error_common` and `tjcl_error_format` to your dependencies.

```rust
use tjcl_error_common::{ErrorKind, I2cSub};
use tjcl_error_format::ErrorFormat;

let err = ErrorKind::I2C(I2cSub::AddressNack);

// Retrieve static strings directly without heap allocation
let kind_name: &'static str = err.kind_str();     // "I2C"
let detail_name: &'static str = err.detail_str(); // "AddressNack"
let full_name: &'static str = err.full_str();     // "I2C::AddressNack"

// Pass directly to display APIs or loggers without intermediate buffers:
// display.draw_text(full_name);
// defmt::info!("{=str}", full_name);
```

---

### Customization (Bit Layout Modification)

This template defaults to an 8-bit parent / 8-bit child (`u16`) layout. If adapting to 4-bit + 4-bit (`u8`) or automotive DTCs (24/32-bit), modify the corresponding types, shift widths, and bitmask operations in `macros.rs` according to your requirements.

---

### License

This software is released under the **[MIT-0 (MIT No Attribution License)](LICENSE)**.

* **License Overview**:  
  MIT-0 is a license that does not require users to retain copyright notices or license text (no attribution required). You are free to modify and integrate it into commercial or open-source projects.
* **Voluntary Attribution (Honor System)**:  
  While not required, a repository Star or a voluntary credit note (`Tsukuyomi Japan Code Lab`) is always appreciated if this template proves useful in your projects.

---

<!-- Original repository copyright notice -->
Copyright (c) 2026 Tsukuyomi Japan Code Lab
