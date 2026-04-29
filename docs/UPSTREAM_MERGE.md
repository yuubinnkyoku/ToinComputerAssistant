# Upstream merge notes

## 2026-04-21: Gemini backend extension

- 目的: upstream の OpenAI 実装を壊さずに Gemini AI Studio バックエンドを追加する。
- 方針: 既存 `src/llm/client.rs` は温存し、fork 固有機能は `src/llm/gemini/` と `src/llm/router.rs` に隔離。
- 既存挙動: デフォルトモデルは従来どおり OpenAI 系で、Gemini は明示的にモデル選択したときのみ使用。
- 追加機能:
  - Gemini backend (`gemini-3.0-flash`, `gemini-3.0-pro`, `gemini-3.1-pro`)
  - `gemini-auto` フェイルオーバー（設定順に順次試行。既定順は quality 優先）
  - Gemini tool-calling ループ
  - 画像付き入力の Gemini 変換（URL→inline data）
  - Gemini `google_search` tool の opt-in

## Conflict-prone files

- `src/app/config.rs`（モデル列挙・環境変数）
- `src/discord/events.rs`（推論呼び出し箇所）

他は追加モジュール中心のため、upstream 追従時の衝突可能性は低め。
