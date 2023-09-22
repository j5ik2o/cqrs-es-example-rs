# read-model-updater

write-api-serverで発生するドメインイベントを使って、read-api-serverが利用するリードモデルを更新するプロセス。

## Dockerfile

- Dockerfile: 本番用
- Dockerfile.local: ローカル用

## エントリポイント

- `src/main.rs`: 本番用
- `bin/local.rs`: ローカル用
