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
- 注意: 既存 tool の JSON Schema は OpenAI 側と共有のまま維持し、Gemini に送る直前だけ `src/llm/gemini/schema.rs` で Gemini FunctionDeclaration 用の subset に正規化する。

## 2026-05-05: NVIDIA NIM backend extension

- 目的: upstream の OpenAI 実装を壊さずに、OpenAI互換APIを公開する NVIDIA NIM を opt-in で利用可能にする。
- 方針: NIM 固有の Chat Completions adapter は `src/llm/nim/` に隔離し、既存 `src/llm/client.rs` は変更しない。
- 既存挙動: デフォルトモデルと OpenAI 経路は従来どおり。NIM は `/model set nim-default` を選んだときのみ使用。
- 追加設定:
  - `NIM_BASE_URL` (NIM 使用時は必須。NVIDIA hosted API 例: `https://integrate.api.nvidia.com/v1`)
  - `NIM_API_KEY` (NIM 使用時は必須)
  - `NIM_DEFAULT_MODEL` (既定: `nvidia/llama-3.1-nemotron-nano-8b-v1`)
- 注意: NVIDIA hosted API は `POST /v1/chat/completions` を使う。NIM 経路では OpenAI built-in web search と Responses API の `reasoning` field を送らない。

## 2026-05-06: Windows MSVC dev build profile

- 目的: VOICEVOX 依存を含む Windows/MSVC の debug build で linker が PDB 上限 (`LNK1318`) に当たるのを避ける。
- 方針: `cargo run` が使う dev profile の debug info だけを無効化する。release profile と runtime 挙動は変更しない。

## 2026-10-29: upstream v0.1.13 まで追従（コンパクション等の大規模再編への移植）

- 取り込み: upstream/main の 8 コミット（v0.1.9 → v0.1.13）。context compaction、`/status`、コマンド再編、モデル定義の刷新（`src/llm/models.rs` へ移動、GPT-5.6/6 系・画像生成モデル追加）、VC 自動切断、TCP モニタ等。
- 移植ポイント:
  - `Models` 列挙は upstream の `src/llm/models.rs` を正とし、fork の Gemini 3 種・`gemini-auto`・`nim-default` を追加（`list` / `from_name`（2.5 系エイリアス含む）/ `rate_cost` / `Display` / `is_gemini`）。`ModelFeatures.cached_tokens` は OpenAI 経路専用として Gemini/NIM では無効化。
  - `src/app/config.rs` は upstream の構造（cron 用レート制限、compaction 設定、`monitor_tcp_bind`、typo 修正済み `rate_limit_window_size`）へ `GeminiConfig` / `NimConfig` を合流。
  - 推論呼び出し箇所は upstream 再編後の `src/discord/responses.rs` が正。`router::generate_response_by_model(ob_ctx, lm_context, GenerateResponseOptions)` が新しい署名で分岐し、Gemini/NIM 以外は upstream の `LMClient::generate_response` へ素通しする。`src/discord/events.rs` は upstream の薄いディスパッチャをそのまま採用。
  - `LMContext` / `Role` は `src/llm/context.rs`、`LMTool` / `ToolExecutionContext` は `src/llm/tool.rs` へ移動済み。Gemini の tool 実行は `execute_with_context` に変更し、`model-control` 等の context 必須 tool が Gemini 経由でも動くようにした。Gemini mapper は OpenAI 経路が保存した output メッセージ（assistant）も `model` ロールとして引き継ぐ。
  - upstream 廃止の delta streaming に合わせ、Gemini/NIM adapter から `delta_mpsc` を削除（状態通知は `state_sender` のみ）。
  - `Cargo.toml` で両側が独立に追加した `base64` が重複したため upstream 側（`0.22`、lock は 0.22.1）に一本化。
- 既知の制約:
  - 応答途中の `model-control` による Gemini/NIM への切替は、その応答内では発効せず次回応答から router が振り分ける（OpenAI 経路のラウンド内切替とは異なる）。
  - context compaction の要約は upstream どおり OpenAI の `CONTEXT_SUMMARY_MODEL` 固定で、Gemini/NIM 選択中も変わらない。
  - upstream の未マージ branch `play-music`（YouTube 音楽再生、1 コミット）は main 追従の対象外として取り込んでいない。
- 検証: `cargo check` 成功、`cargo test` 18 件全通過。`cargo clippy --all-targets --all-features` は警告 11 件のみで、いずれも upstream / fork の既存コード分（今回の移植で新規に増えた警告はなし）。検証環境では voicevox_core の git 取得がプロキシ経由で詰まったため、同一リビジョンの tarball を path patch した検証用コピーで実施した。kanalizer の build script が行う Hugging Face からのモデル取得は検証環境の証明書検証で失敗するため、モデルファイルを事前配置した（いずれも環境要因でありコード起因ではない）。

## Conflict-prone files

- `Cargo.toml`（Windows/MSVC dev build の debug info 抑制、`base64` の重複追加）
- `src/app/config.rs`（環境変数。モデル列挙は `src/llm/models.rs` 側へ移動済み）
- `src/llm/models.rs`（upstream のモデル定義へ fork の Gemini/NIM バリアントを追加）
- `src/llm/router.rs`（upstream の `GenerateResponseOptions` 署名への追従）
- `src/discord/responses.rs`（推論呼び出し箇所。旧 `src/discord/events.rs` から移転）

他は追加モジュール中心のため、upstream 追従時の衝突可能性は低め。
