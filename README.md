# Looker

## Overview
外付けSSDのフォルダ構造を最適化するためのCLIアプリケーション
ドライブ全体を探索し,推奨配置を提案,
また,自動実行ができる形式にする

フォルダ運用は開発者の経験から,
目的からの探索アプローチを前提にして最適化する

特にWeb開発とデザイン/映像を扱う開発者を対象とする

## 機能

- **対話型メニュー**: リッチなUIで直感的に操作可能
- **フォルダ構造の検証**: 標準フォルダ構造に適合しているかチェック
- **ドライブスキャン**: ドライブ全体を再帰的に探索し、ファイルを分類
- **推奨配置提案**: ファイルの種類に基づいて最適な配置先を提案
- **命名規則チェック**: プロジェクトファイルとrecordファイルの命名規則を検証
- **自動実行**: 推奨配置を自動実行（ドライラン/実行モード対応）
- **標準構造作成**: 標準フォルダ構造を自動作成
- **Recordフォルダ整理**: 日付ベースで自動分類、命名規則に従ってリネーム・移動
- **デフォルトドライブ自動検出**: 実行ファイルが存在するドライブを自動選択

## インストール

### ビルド方法

```bash
# リポジトリをクローン
git clone <repository-url>
cd Looker

# リリースビルド
cargo build --release

# 実行ファイルは target/release/Looker.exe (Windows) または target/release/Looker (Linux/macOS) に生成されます
```

## 使用方法

### 対話型メニュー（推奨）

引数なしで実行すると、リッチなUIの対話型メニューが表示されます。

```bash
# 対話型メニューを起動
cargo run --release

# またはビルド済みの実行ファイル
./target/release/Looker.exe
```

メニューから以下の操作が可能です：

1. **フォルダ構造を検証** - 標準フォルダ構造に適合しているかチェック
2. **ドライブをスキャン** - ドライブ全体を探索し、ファイルを分類
3. **推奨配置を提案** - ファイルの最適な配置先を提案
4. **標準フォルダ構造を作成** - 標準フォルダ構造を自動作成
5. **推奨配置を実行** - 推奨配置に基づいてファイルを自動移動
6. **Recordフォルダを整理** - recordフォルダ内のファイルを日付ベースで整理
7. **終了** - アプリケーションを終了

### CLIモード（従来の方法）

引数を指定すると、従来のCLIモードで動作します。

```bash
# ヘルプを表示
cargo run --release -- --help

# サブコマンドのヘルプを表示
cargo run --release -- <command> --help
```

### フォルダ構造の検証

指定したルートディレクトリが標準フォルダ構造に適合しているかチェックします。

```bash
# CLIモード
cargo run --release -- validate --root D:\

# 対話型メニューでは、デフォルトで実行ファイルのドライブが選択されます
```

### ドライブのスキャン

指定したパスを再帰的にスキャンし、ファイルをカテゴリ別に分類します。

```bash
# CLIモード
cargo run --release -- scan --path D:\
```

### 推奨配置の提案

スキャンしたファイルに対して、最適な配置先を提案します。

```bash
# CLIモード
cargo run --release -- recommend --path D:\ --root D:\
```

### 標準フォルダ構造の作成

標準フォルダ構造を自動作成します。`--dry-run`オプションで実際には作成せずに確認できます。

```bash
# ドライラン（確認のみ）
cargo run --release -- init --root D:\ --dry-run

# 実際に作成
cargo run --release -- init --root D:\
```

### 推奨配置の実行

推奨配置に基づいてファイルを自動移動します。`--dry-run`オプションで実際には移動せずに確認できます。

```bash
# ドライラン（確認のみ）
cargo run --release -- execute --path D:\ --root D:\ --dry-run

# 実際に実行
cargo run --release -- execute --path D:\ --root D:\
```

### Recordフォルダの整理

`0_inbox/record`フォルダ内のファイルを自動整理します。

- **日付ベースの分類**: ファイルの更新日時に基づいて自動分類
  - 1年以上前: `YYYY/MM/` フォルダに配置
  - 1年以内: `MM/` フォルダに配置
- **命名規則の適用**: 命名規則に従っていないファイルを自動リネーム
- **自動移動**: 適切なフォルダに自動移動

```bash
# 対話型メニューから「Recordフォルダを整理」を選択
# またはCLIモード（今後実装予定）
```

### 命名規則のチェック

ファイル名が命名規則に適合しているかチェックします。

```bash
# プロジェクトファイルのチェック
cargo run --release -- check-naming --path "20240101_project_item_option.txt"

# recordファイルのチェック
cargo run --release -- check-naming --path "20240101120000_screen-capture.png" --is-record
```

## ファイル分類ルール

アプリケーションは以下のルールに基づいてファイルを分類し、推奨配置を提案します：

- **動画ファイル** (mp4, avi, mov, mkv, wmv, flv, webm, m4v) → `2_assets/footage`
- **画像ファイル** (jpg, jpeg, png, gif, bmp, webp)
  - イラスト関連 → `2_assets/illust`
  - グラフィック/デザイン関連 → `2_assets/graphic`
  - その他 → `2_assets/photo`
- **音声ファイル** (mp3, wav, flac, aac, ogg, wma, m4a)
  - BGM/音楽関連 → `2_assets/bgm`
  - その他 → `2_assets/sfx`
- **ドキュメントファイル** (pdf, doc, docx, xls, xlsx, ppt, pptx, txt, md) → `3_docs`
- **アーカイブファイル** (zip, rar, 7z, tar, gz) → `9_archive`
- **その他** → `0_inbox`

## Rule
### フォルダ構造
フォルダ運用は以下のフォルダ構造に従う

```
root/
├ 0_inbox/
│ ├ downloads/
│ └ record/
│   ├ screen capture/
│   │ ├ YYYY/YYYYMM/ #去年以前
│   │ └ YYYYMM/ #今年
│   ├ screen record/
│   │ ├ YYYY/YYYYMM/ #去年以前
│   │ └ YYYYMM/ #今年
│   └ voice record/
│     ├ YYYY/YYYYMM/ #去年以前
│     └ YYYYMM/ #今年
├ 1_projects/[category]/[project]/ #プロジェクト単位でアクセスする前提のファイル
├ 2_assets/    #メディアファイル
│ ├ footage/[category]/
│ ├ graphic/[category]/
│ ├ photo/[category]/
│ ├ illust/[category]/
│ ├ bgm/[category]/
│ └ sfx/[category]/
├ 3_docs/  #複数回アクセスする前提のファイル
│ ├ plofile/
│ ├ collection/
│ ├ class/
│ ├ cclub/
│ ├ guide/
│ ├ family/
│ ├ icon/
│ └ meme/
├ 4_apps/[app_name]/[app_folder]/ #アプリケーション
├ 5_gallery/ #完成品メディアファイルをlinkファイル
├ 9_archive/ #使用する予定の無いファイルやフォルダをzipファイル

```

### 命名規則
プロジェクトファイル名を決定する場合は以下の規則に従う

```
YYYYMMDD_[project]_[item]_[option].[extension]
```

例: `20240101_website_homepage_draft.png`

### record内

```
YYYYMMDDHHMMSS_[screen-capture/screen-record/voice-record].[extension]
```

例: `20240101120000_screen-capture.png`

### Recordフォルダの整理ルール

Recordフォルダ内のファイルは、以下のルールに従って自動整理されます：

- **日付による分類**:
  - **去年以前のファイル**: `YYYY/YYYYMM/` フォルダに配置（例: `2023/202312/`）
  - **今年のファイル**: `YYYYMM/` フォルダに配置（例: `202412/`）
- **命名規則の適用**:
  - 命名規則に従っていないファイルは、ファイルの更新日時を基に自動リネーム
  - 形式: `YYYYMMDDHHMMSS_[screen-capture/screen-record/voice-record].[extension]`
  - 例: `20240101120000_screen-capture.png`
- **自動移動とフォルダ作成**:
  - 適切なフォルダが存在しない場合は自動作成
  - ファイルを適切なフォルダに自動移動
  - 各recordタイプ（screen capture, screen record, voice record）ごとにフォルダが自動作成されます

## 特徴

### リッチなCLI演出

- **カラフルな出力**: 成功（緑）、エラー（赤）、警告（黄）、情報（青）で色分け
- **視覚的な記号**: ✓, ✗, ⚠, ℹ, ▶ などの記号で状態を表示
- **ローディングアニメーション**: 処理中はスピナーで進捗を表示
- **セクション区切り**: 見やすい区切り線とセクションタイトル

### デフォルトドライブ自動検出

- 実行ファイルが存在するドライブを自動的にデフォルトとして選択
- 対話型メニューでは、カスタムパスを指定することも可能

### Recordフォルダ自動整理

- **日付ベースの分類**: ファイルの更新日時に基づいて自動分類
  - **去年以前のファイル**: `YYYY/YYYYMM/` フォルダに配置（例: `2023/202312/`）
  - **今年のファイル**: `YYYYMM/` フォルダに配置（例: `202412/`）
- **自動リネーム**: 命名規則に従っていないファイルを自動的にリネーム
- **フォルダ自動作成**: 必要なフォルダを自動的に作成（各recordタイプごと）

## 開発

### 依存関係

- `clap`: CLIパーサー
- `anyhow`: エラーハンドリング
- `walkdir`: 再帰的なディレクトリ探索
- `regex`: 正規表現による命名規則チェック
- `chrono`: 日時処理
- `inquire`: 対話型メニュー
- `colored`: カラフルなCLI出力
- `indicatif`: プログレスバーとローディングアニメーション

### プロジェクト構造

```
src/
├── main.rs          # CLIエントリーポイント（対話型/CLIモード切り替え）
├── structure.rs     # フォルダ構造の定義と検証
├── scanner.rs       # ドライブスキャンとファイル分類
├── recommender.rs   # 推奨配置の提案
├── naming.rs        # 命名規則のチェック
├── executor.rs      # 自動実行機能
├── record_manager.rs # Recordフォルダの管理と整理
├── ui.rs            # リッチなCLI出力
├── menu.rs          # 対話型メニューシステム
└── utils.rs         # ユーティリティ関数（デフォルトドライブ取得など）
```

## ライセンス

このプロジェクトのライセンス情報をここに記載してください。

