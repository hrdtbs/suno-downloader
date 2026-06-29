# Suno Downloader

Suno のマイライブラリから WAV ファイルをローカルに同期する Windows 向けデスクトップアプリです。

## 機能

- Suno ライブラリの WAV 同期（suno-cli 互換）
- Chrome 拡張機能による認証（推奨）
- 手動 JWT 入力（フォールバック）
- フォルダ整理（flat / month / week / month-week）
- 期間フィルタ（since: `7d`, `1w` など）

## 開発

```bash
pnpm install
pnpm prepare
pnpm tauri dev
```

## ビルド

```bash
pnpm tauri build
```

## 認証

1. アプリ起動時にローカルトークンサーバー（`127.0.0.1:38946`）が起動します
2. `chrome-extension/` を Chrome に読み込みます
3. [suno.com/me](https://suno.com/me) にログインすると自動でトークンが保存されます

## データ保存先

| データ               | パス                                      |
| -------------------- | ----------------------------------------- |
| セッション           | `%APPDATA%/suno-downloader/session.json`  |
| 設定                 | `%APPDATA%/suno-downloader/settings.json` |
| ローカルインデックス | `{出力dir}/.suno-cli-index.json`          |

suno-cli のセッション・インデックス形式とも互換です。

## ライセンス

MIT
