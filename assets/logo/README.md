# Qi Logo Assets (Otter Edition)

Qiプログラミング言語のロゴファイル集です。カワウソをモチーフにしたマスコットデザイン。

## 🎨 ファイル構成

### メインロゴ（SVG - 推奨）

- **`qi-logo-full.svg`** - フルロックアップ（カワウソ + "Qi" + 流れる尾）
  - README タイトル、ヒーローバナー用
  - **推奨用途**: プロジェクトREADME、ドキュメントのトップ

- **`qi-logo-horizontal.svg`** - コンパクトな横レイアウト
  - ウェブサイトヘッダー用

- **`qi-icon.svg`** - アイコンのみ（円形背景付き）
  - アバター、アプリアイコン用

- **`qi-mark.svg`** - ミニマルマーク（`|>` 風の流れる尾）
  - ファビコン、小さいスペース用

- **`qi-light.svg` / `qi-dark.svg`** - ライト/ダークモード用バリアント

### PNG エクスポート（高解像度）

- **`qi-logo-full-[1024|512|256|128|64|48|32].png`** - すぐに使えるラスター画像
  - 1024px: 高解像度印刷、大型ディスプレイ
  - 512px: 標準的なREADME表示
  - 256px以下: サムネイル、リスト表示

### ファビコン

- **`favicon.ico`** - マルチサイズファビコン（標準形式）

---

## 🎨 カラーパレット

| 色名 | HEX | 用途 |
|------|-----|------|
| **Indigo** | `#1B2B34` | メインカラー（ダーク） |
| **Gold** | `#E3B23C` | アクセントカラー（ゴールド） |
| **Brown** | `#C6A27C` | サブカラー（ブラウン） |
| **Cream** | `#F2E3CF` | 背景カラー（クリーム） |

---

## 📖 使用方法

### README.mdでの使用（SVG推奨）

```markdown
<p align="center">
  <img src="./assets/logo/qi-logo-full.svg" alt="Qi Logo" width="400">
</p>
```

### HTMLでの使用

```html
<!-- SVG版（スケーラブル） -->
<img src="./assets/logo/qi-logo-full.svg" alt="Qi Logo" width="400">

<!-- PNG版（固定サイズ） -->
<img src="./assets/logo/qi-logo-full-512.png" alt="Qi Logo" width="400">
```

### ファビコン設定

```html
<link rel="icon" href="/assets/logo/favicon.ico">
```

### ダークモード対応

```html
<picture>
  <source srcset="./assets/logo/qi-dark.svg" media="(prefers-color-scheme: dark)">
  <img src="./assets/logo/qi-light.svg" alt="Qi Logo" width="400">
</picture>
```

---

## 💡 Tips

- **SVGを優先**: ドキュメントやウェブサイトではSVGを使用（完全にスケール可能）
- **アイコン用途**: `qi-icon.svg` をソーシャルアバターやアプリアイコンに
- **尾のデザイン**: カワウソの尾がQiのパイプライン演算子 `|>` を表現
- **PNG使用時**: 表示サイズの2倍の解像度を選択（Retina対応）

---

## 📐 推奨サイズ

| 用途 | 推奨サイズ | ファイル |
|------|----------|---------|
| README表示 | 400-600px幅 | `qi-logo-full.svg` |
| ヘッダー | 200-300px幅 | `qi-logo-horizontal.svg` |
| アイコン | 32-512px | `qi-icon.svg` |
| ファビコン | 32x32px | `favicon.ico` |
| SNSカード | 1200x630px | （要作成） |

---

## ✅ 使用ガイドライン

### 推奨される使用方法
- プロジェクトのREADME、ドキュメント
- プレゼンテーション、スライド
- ブログ記事、技術記事での言及
- 教育目的での使用

### ⚠️ 注意事項
- ロゴの改変は避けてください
- 縦横比を保ってください
- 背景色に応じて適切な版（light/dark）を選択してください
- 小さいサイズでは `qi-mark.svg` を推奨

### ❌ 禁止事項
- 商用利用（要相談）
- ロゴを使った誤解を招く表現
- Qiプロジェクトとの公式関係を示唆する使用（承認なし）

---

## 📄 ライセンス

ロゴデザイン: © 2025 Qi Programming Language

使用条件: 未定（プロジェクト公開時に決定予定）

---

## 📝 更新履歴

- 2025-01-15: ロゴパック (Otter Edition) 追加
- 2025-01-15: ロゴアセット構造作成
