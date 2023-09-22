# モジュール構成

基本的にクリーンアーキテクチャを踏襲したレイヤー構造を採用します。

- command: コマンド側
    - domain: ドメイン層
    - interface-adaptor-if: コマンド側のインターフェイスアダプタ層のI/F。
    - interface-adaptor-impl: コマンド側のインターフェイスアダプタ層の実装。Controller, Presenter, Gatewayなど。
    - processor: ユースケース層(コマンドプロセッサ)
- query: クエリ側
    - interface-adaptor: クエリ側のインターフェイスアダプタ層。ほとんどがGraphQLの記述になります。I/Fと実装に分離はされません。
- infrastructure: インフラストラクチャ層(汎用的な技術基盤)。永続化責務の実装はこちらに配置しないでください。

## クリーンアーキテクチャに関する参考資料

まずこのあたりを読むとよい

- [クリーンアーキテクチャ完全に理解した](https://gist.github.com/mpppk/609d592f25cab9312654b39f1b357c60)

より詳しい知識はこちらを参考にしてください

- [Clean Architecture 達人に学ぶソフトウェアの構造と設計](https://amzn.to/47sa385)
- [クリーンアーキテクチャ(The Clean Architecture翻訳)](https://blog.tai2.net/the_clean_architecture.html)

今回の実装では上記の考え方を参考にしながら、独自の工夫を取り入れている部分があります。

注意点としては、クリーンアーキテクチャのEntitiesの部分がDDDでいうドメイン層に相当するので読み替えてください。
クリーンアーキテクチャでは、CQRS/Event Sourcingは説明としてでてこないので、組み合わせたときにどのように設計があるべきかは考える必要があります。