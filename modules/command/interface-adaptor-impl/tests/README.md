# `tests`とはなにか

Rust流儀としては、単体テストは、通常実装ファイルと同じ場所に書きますが、結合テストは　`tests`に書きます。

`tests/commmand_processor.rs`が結合テストですが、これ以外は単体テストです。
なのに、実装ファイル側に書いていないのは`common.rs`を利用したテストの置き場を実装ファイル側に配置するのが面倒＆時間がなかったからです。

ということで、なぜこれだけ`tests`に配置しているのだろう？という疑問はあると思いますが、実装都合で`tests`にまとめているという理解でお願いします。
綺麗に整理するなら、結合テストだけ`tests`に残して、他は書く実装ファイルの中にテストを移動するとよいです。

## `tests/commmand_processor.rs`をなぜ`interface-adaptor-impl`に配置するのか

`commmand-processor`の`GroupChatCommandProcessor`はリポジトリを利用していますが、リポジトリの具体的な実装型に依存していません。つまり、`GroupChatRepository`トレイトという抽象型を利用したロジックになっています。なぜそうなるかというと、モジュールの依存関係が関係しています。`GroupChatRepository`は`interface-adaptor-if`に定義されていて、`commmand-processor`から依存できるようにしています。が、`interface-adaptor-impl`は実装なので`commmand-processor`から依存できないものとしています。これはクリーンアーキテクチャの原則に従っているのでこのようになっています。

つまり、`commmand-processor`の`GroupChatCommandProcessor`は一切実装しらないので、`commmand-processor`でリポジトリを使ったテストが書けません。なので、`interface-adaptor-impl`の結合テストで、実際のリポジトリの実装を使ってテストを行っています。
