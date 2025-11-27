# Looker

Recordフォルダ専用の整理CLIです。  
実際の運用で必要だった「`0_inbox/record` 以下のスクリーンショット/録画/音声を自動で振り分け、命名規則を揃える」ことだけに機能を絞り、ビルド時間と操作手順をミニマルにしました。

## できること

- `0_inbox/record` 配下、または任意で指定したrecordフォルダを解析し、必要なフォルダ作成やファイル移動/リネームの計画を生成。
- ファイルの更新日時から `YYYY/YYYYMM/` または `YYYYMM/` フォルダを自動判定。
- 命名規則 (`YYYYMMDDHHMMSS_[screen|voice].ext`) から外れている場合は再命名案を提示し、必要なら実際にリネーム。
- screen/voiceなど録画種別を取り違えているファイルを再分類（`--fast` 指定時はスキップして高速化）。
- `--record-type` で screen-capture / screen-record / voice-record のいずれかに処理対象を絞り込み。
- `--apply` + `--yes` でノンインタラクティブに実行可能（自動整理タスク等に組み込みやすい）。

## セットアップ

```bash
git clone <repository-url>
cd Looker
cargo build --release
```

Windowsでは `target/release/Looker.exe`、macOS/Linuxでは `target/release/Looker` が実行ファイルです。

## 使い方

### 1. ドライランで計画を確認

```bash
# 既定ではカレントディレクトリ直下の 0_inbox/record を解析
looker

# ドライブ直下をルートとして解析し、screen capture のみ表示
looker --root D:\ --record-type screen-capture
```

出力例:

```
Recordフォルダ: D:\0_inbox\record
作成が必要なフォルダ: 2
  - D:\0_inbox\record\screen capture\2023
  - D:\0_inbox\record\screen capture\2023\202309
ファイル操作数: 5
  - [MOVE] D:\0_inbox\record\foo.png -> D:\0_inbox\record\screen capture\2023\202309\20230910103000_screen-capture.png
  ...
--apply を指定すると上記の変更を適用できます。
```

### 2. 変更を適用

```bash
# 確認付きで適用
looker --root D:\ --apply

# CI等、確認なしで適用する場合
looker --root D:\ --apply --yes
```

### 主なオプション

| オプション | 説明 |
| --- | --- |
| `--root <PATH>` | `0_inbox/record` を含むルートディレクトリを指定（既定: `.`） |
| `--record-path <PATH>` | recordフォルダを直接指定。`--root` より優先 |
| `--record-type <TYPE>` | `screen-capture` / `screen-record` / `voice-record` のいずれか。複数指定可 |
| `--fast` | 種別取り違えチェックをスキップし高速化 |
| `--apply` | 計画された変更を実行 |
| `--yes` | 事前確認なしで `--apply` を実行（`-y` も可） |
| `--verbose` | すべてのフォルダ作成・ファイル操作を表示（既定では最大10件までプレビュー） |

## Lint & QA

- ローカル確認: `cargo fmt --all --check` と `cargo clippy --all-targets -- -D warnings`
- GitHub Actions で同じ lint ワークフローを `push` / `pull_request` ごとに実行し、整形漏れや警告をブロックします（`.github/workflows/lint.yml`）

## フォルダ/命名ルール

- record種別: `screen capture`, `screen record`, `voice record`
- 1年より前のファイルは `YYYY/YYYYMM/`、当年分は `YYYYMM/` に配置
- ファイル名: `YYYYMMDDHHMMSS_screen-capture.png` の形式

## 内部構成

- `src/record_manager.rs`: フォルダ解析とアクション生成/適用ロジック
- `src/main.rs`: CLI本体（オプション解析とレポート表示）
- `src/scanner.rs`: WalkDirベースの簡易ファイルスキャナ
- `src/naming.rs`: record向け命名ルール

## ライセンス

このリポジトリに含まれるコードは作者の私用ツールとして公開しています。詳細なライセンスが必要な場合はご連絡ください。
