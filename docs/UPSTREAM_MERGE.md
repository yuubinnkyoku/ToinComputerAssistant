# Upstream merge notes

このファイルは、fork 固有差分の意図を最小限で記録するためのメモです。

## 方針

- upstream (`371tti/Nelfie`) への追従を優先し、既存実装の直接改変は最小限にする。
- fork 固有機能は `src/fork_ext/` に集約し、既存モジュール側は glue code のみに抑える。
- 新機能は設定フラグで opt-in にし、デフォルト挙動は upstream 互換のまま維持する。

## 現在の差分（初期）

- `src/fork_ext/config.rs` を追加し、fork 拡張用の設定読み取りを分離。
- `src/lib.rs` と `src/app/config.rs` では `fork_ext` 設定を保持するための最小変更のみ実施。
- `src/fork_ext/text_pipeline.rs` を追加し、TTS 前処理（`www` / `草` 系の軽量正規化）を opt-in で提供。
- `src/fork_ext/tts_preprocessor.rs` を追加し、`Noop` / `Fork` の差し替えを provider 的に切り替える。
- `src/voice/system.rs` は glue code として preprocessor trait を呼び出すだけにし、分岐を隔離する。

## 今後のルール

- dendenmushi-rust の機能は「直接移植」ではなく、Nelfie の構造に合わせて機能単位で小分け導入する。
- upstream で衝突しやすいファイル（`app/context.rs`, `app/config.rs`, `discord/commands.rs`）の変更は末尾追加・局所変更を徹底する。
