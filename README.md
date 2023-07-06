# cqrs-es-example-rs

ステータス:実装中

## コンポーネント構成

- Write API (WIP)
    - 書き込み専用のWeb API
- Read Model Updater
    - ジャーナルを基にリードモデル構築するLambda
- Read API
    - 読み込み専用Werb API

## ミドルウェア構成

- DynamoDB(Journal, Snapshot)
- DynamoDB Streams
- Lambda
- RDS(Aurora)
- Redis(PubSub用途)

## ストリーム構成

Client -> Write API(Web API) - Event -> DynamoDB,DynamoDB Streams -> RMU(Lambda) - SQL/PubSub -> DynamoDB(Read Model) -> Read API(GraphQL Query/Subscription) -> Client

