# query-interface-adaptor

## 概要

このモジュールはクエリのインターフェースアダプターを提供します。

- Controllerの実装は、GraphQL用の`graphql_handler`だけです
- Presenterの実装はありません
- Gatewayの実装は`gateways.rs`を参照してください。
- GraphQLではユースケースの実装もありません。(GraphQLのクエリ自体がユースケースを兼ねているためです)

なので、GraphQLを使った場合はほとんどの実装が省力化できることになります。