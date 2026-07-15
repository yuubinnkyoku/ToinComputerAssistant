# AGENTS.md

このファイルはリポジトリ全体に適用される、コーディングエージェント向けの作業ガイドです。

## プロジェクト概要

Nelfie（このフォークでは Toin Computer Assistant）は、Discord 上で会話、ツール実行、VOICEVOX 読み上げを提供する Rust 製 Bot です。Rust edition 2024 を使用し、非同期ランタイムは Tokio、Discord クライアントは Serenity / Poise です。

## 主なディレクトリ

- `src/app`: 環境変数による設定、共有コンテキスト、cron の管理と実行
- `src/discord`: slash / prefix command と Discord イベント処理
- `src/llm`: OpenAI、Gemini、NVIDIA NIM のクライアント、モデルルーティング、会話状態
- `src/llm/tools`: LLM から呼び出す Discord、cron、LaTeX、VOICEVOX などのツール
- `src/voice`: VOICEVOX ランタイム、音声生成、読み上げキュー、辞書・テキスト処理
- `fonts`: LaTeX 描画用フォント。再配布時は `THIRD_PARTY_LICENSES.md` を維持する
- `voicevox_core`: 実行時アセット。Git 管理対象外
- `data`: 実行時に生成される状態。Git 管理対象外

## 作業方針

- 変更前に関連モジュールと既存テストを読み、既存の設計と命名に合わせる。
- 要求に直接関係しないリファクタリングや一括整形は行わない。
- Discord、LLM、音声処理の非同期経路では、ブロッキング I/O や長時間の同期処理を追加しない。
- 共有状態を変更するときは、既存の `Arc`、ロック、チャネルの使い方を踏襲し、ロックを保持したまま `.await` しない。
- 新しい LLM バックエンドやモデル分岐は、呼び出し元へ直接条件分岐を増やす前に `src/llm/router.rs` へ統合できるか確認する。
- 新しい LLM ツールを追加した場合は、`src/llm/tools/mod.rs` への登録と、利用可能なツール一覧への組み込みを忘れない。
- slash command の追加・変更時は、Poise の登録箇所、権限、prefix command との整合性を確認する。
- ユーザー向け設定を追加した場合は、`src/app/config.rs` と README の `.env` 例を同時に更新する。
- 永続化形式や `global-dict.json` を変更する場合は、既存データとの後方互換性を考慮する。

## 秘密情報と生成物

- `.env`、Discord token、API key、ユーザーの会話内容をコミットしない。
- ログやエラーに token、認証ヘッダー、完全な API 応答を出さない。
- `target/`、`data/`、`voicevox_core/`、`config.json` は生成物またはローカル状態として扱う。
- `Cargo.lock` はアプリケーションの再現可能なビルドに必要なため、依存関係変更時は更新してコミットする。

## ビルドと検証

変更範囲に応じて、次のコマンドを実行する。

```powershell
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

- Rust ファイルを編集した場合は `cargo fmt -- --check` で確認する。ただし、このリポジトリでは全体整形が常時強制されていないため、無関係な既存コードまで整形しない。
- 小さな変更では関連テストを先に実行してよい。例: `cargo test app::cron::tests`。
- テストは対象コードの近くにある `#[cfg(test)] mod tests` へ追加する既存形式を優先する。
- 外部 API、Discord 接続、VOICEVOX モデルを必要とする確認は、資格情報や大容量アセットが利用可能な場合のみ行い、未実施理由を報告する。
- Windows の MSVC ビルドで `stdint.h` などの標準ヘッダーが見つからない場合は、Visual Studio Build Tools の C++ ワークロードと Windows SDK が有効な Developer PowerShell で再実行する。コード不具合と決めつけない。
- VOICEVOX 関連の初回ビルドは ONNX Runtime や依存アセットの取得に時間がかかる。

## コード品質

- エラーを黙って握りつぶさず、呼び出し側が処理できる形で返すか、利用者が対処できる文脈を付けてログへ記録する。
- 本番経路で `unwrap()` や `expect()` を新規に使わない。起動時に必須条件を検証する場合や、テスト内では理由が明確なら許容する。
- 公開 API、設定項目、複雑な並行処理には「何をしているか」より「なぜ必要か」が分かるコメントを付ける。
- ユーザー向け Discord メッセージと README は、既存の日本語表現に合わせる。
- 依存関係を追加する前に、標準ライブラリまたは既存依存で実装できないか確認する。

## 完了時の報告

- 変更した機能と主要ファイルを簡潔にまとめる。
- 実行した検証コマンドと結果を記載する。
- 実行できなかった検証、既知の制約、必要な環境設定があれば明記する。
- ユーザーから明示的に依頼されていない限り、commit、push、release は行わない。
