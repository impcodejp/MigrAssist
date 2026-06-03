# MigrAssist — 給与・勤怠システム突合ツール

給与・勤怠システム導入時に、現行システム（他社システム）と新システム（MJSシステム）がそれぞれ出力したCSVファイルを読み込み、指定した項目同士を1レコードずつ突合して差異を検出するデスクトップツールです。

## 機能概要

- MJS・他社システムそれぞれのCSVファイルを読み込み（SJIS / UTF-8 / UTF-8 BOM 対応）
- ヘッダー名の完全一致による列の自動マッピング
- 列ごとに許容誤差を設定した突合処理
- 突合結果をCSV（サマリー・明細）として出力
- 突合設定の保存・読み込み（JSON形式）

## 技術スタック

| 区分 | 技術 |
|------|------|
| フレームワーク | [Tauri 2](https://v2.tauri.app/) |
| フロントエンド | React 19 + TypeScript 5 |
| バックエンド | Rust |
| ビルドツール | Vite 7 |

## 対応OS

Windows 11

## 開発環境のセットアップ

### 前提条件

- [Node.js](https://nodejs.org/) 18以上
- [Rust](https://www.rust-lang.org/tools/install)（rustupでインストール）
- [Tauri CLI の前提条件](https://v2.tauri.app/start/prerequisites/)（Microsoft C++ Build Tools など）

### 手順

```bash
# リポジトリをクローン
git clone <repository-url>
cd MigrAssist

# フロントエンド依存パッケージのインストール
npm install

# 開発サーバー起動
npm run tauri dev
```

## ビルド

```bash
# インストーラー（NSIS形式）を生成
npm run tauri build
```

ビルド成果物は `src-tauri/target/release/bundle/nsis/` に出力されます。

## プロジェクト構成

```
MigrAssist/
├── src/                    # フロントエンド（React/TypeScript）
│   ├── api/tauri.ts        # Tauriバックエンドへの呼び出しをまとめたAPI層
│   ├── components/         # UIコンポーネント
│   │   ├── FileInputSection.tsx  # ファイルパス入力エリア
│   │   ├── MappingTable.tsx      # 突合設定テーブル
│   │   └── ProgressBar.tsx       # 進捗バー
│   ├── hooks/
│   │   └── useAppState.ts  # アプリ全体の状態管理フック
│   ├── types/index.ts      # 型定義
│   └── App.tsx             # メイン画面レイアウト
└── src-tauri/src/          # バックエンド（Rust）
    ├── commands.rs         # Tauriコマンドハンドラ（フロントエンドとの境界）
    ├── comparison.rs       # 突合処理のコアロジック
    ├── csv_reader.rs       # CSV読み込み・文字コード変換
    ├── config.rs           # 突合設定の保存・読み込み
    ├── types.rs            # 共有型定義
    └── lib.rs              # Tauriアプリの初期化
```

## 出力ファイル

実行すると出力フォルダに以下の2ファイルが生成されます（ファイル名にはタイムスタンプが付与されます）。

| ファイル名 | 内容 |
|-----------|------|
| `サマリー比較_yyyymmddhhmm.csv` | 列ごとの不一致件数（0件は `〇`）および片側のみ存在レコード数 |
| `明細比較_yyyymmddhhmm.csv` | 不一致があった列の詳細（キー値・MJS値・他社値・差異） |

どちらもExcelで開けるよう UTF-8 BOM 付きで出力されます。

## ライセンス

社内利用限定。
