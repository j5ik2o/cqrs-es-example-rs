# Repository Guidelines

## プロジェクト構成とモジュール
- ルートは Cargo ワークスペースで、`applications/` が実行可能バイナリを持つ；`write-api-server` は GraphQL Mutation、`read-api-server` は Query、`read-model-updater` はリードモデル更新ロジックを提供する。
- ドメインロジックは `modules/command`、読み取り系は `modules/query`、更新ワーカは `modules/rmu` に集約し、再利用可能なインフラ抽象は `modules/infrastructure` で管理する。
- 開発者向け資料は `docs/`、運用スクリプトや Docker/E2E 用ユーティリティは `tools/` に配置される；Lambda 用成果物は `dist/lambda/` に生成される。環境変数テンプレートは `common.env.default` を参照し、編集後は `common.env` を Git 管理から除外したまま保持する。
- モデル固有のテストコードは各 `src` 内の `mod tests` として共存させ、共有フィクスチャは必要に応じて `modules` 配下へ配置する。

## ビルド・テスト・開発コマンド
- 作業開始時に `cp common.env.default common.env` を実行し、接頭辞や AWS 向け設定を更新する。
- RDB 接続が必要な処理の前に `makers docker-compose-up-db` でローカル DB を起動する（`makers` は `cargo make` のラッパー）。
- ビルドは `makers build`、フォーマットは `makers fmt` または `cargo +nightly fmt` を利用し、GraphQL API は `cargo run -p write-api-server --bin write-api-server` などで個別に起動できる。
- DynamoDB Streams 経由のリードモデル更新を確認したい場合は `makers build-read-model-updater-lambda` で Lambda 用 ZIP を生成し、`makers deploy-read-model-updater-localstack` で LocalStack にデプロイする。
- テストは `makers test` を基準とし、並列数を 1 に固定した `RUST_TEST_THREADS=1 cargo test` での再現性を保つ；E2E 確認が必要な場合は `makers verify-group-chat` や `tools/e2e-test/verify-group-chat.sh` を使用する。

## コーディングスタイルと命名
- Rust コードは 2 スペースインデント・120 文字幅（`rustfmt.toml` 参照）を厳守し、PR 前に必ず `cargo +nightly fmt` を実行する。
- モジュール名とファイルは `snake_case`、型とトレイトは `UpperCamelCase`、定数は `SCREAMING_SNAKE_CASE` を採用する。
- クロスモジュール API にはインターフェイス（trait）を定義し実装を分離する既存パターンに従い、GraphQL 定義は `applications/write-api-server/bin/export-*` 系バイナリで更新する。
- 新規 SQL を追加した場合は `makers build` を走らせて `.sqlx/*.json` を再生成し、差分をコミットする。

## テストガイドライン
- コンポーネントテストは `testcontainers` 依存のため Docker を常時起動し、テスト前後にリソースが解放されるか確認する。
- シナリオテストは GraphQL 経由でグループチャットを作成する `makers create-and-get-group-chat` を活用し、再現手順を PR 説明に記載する。LocalStack Lambda 検証時は `makers deploy-read-model-updater-localstack` の実行結果も記す。
- テストケースは「状態-操作-期待値」を明示した関数名（例：`should_create_group_chat_when_members_valid`）とし、`serial_test` のガードが必要なケースでは既存実装を参照して属性を付与する。
- ログ出力が必要な場合は `RUST_LOG=debug` を設定して再実行し、失敗ログを記録する。

## コミットとプルリクエスト
- Git 履歴は Conventional Commits に準拠しており、`chore(deps): Update ... (#541)` のように種別・スコープ・概要・関連 PR 番号を含める；機能追加は `feat`, バグ修正は `fix` を使用する。
- プルリクエストでは目的、主要変更点、影響範囲、実行済みコマンド（例：`makers test`, `cargo +nightly fmt`）をチェックリスト形式で提示し、関連 Issue や GraphQL リクエスト例をリンクする。
- CI の失敗を未解決のまま提出しないこと、依存ライブラリ更新を含む場合は Renovate 設定との重複を避けること、レビューコメントには 24 時間以内に返信することを推奨する。
- マージ前には最新 `main` をリベースし、衝突解消後に再度テストを通す。
