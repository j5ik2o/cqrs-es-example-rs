## `common.env.default`を`common.env`としてコピーし編集する

```shell
$ cp common.env.default common.env
```

環境変数のPREFIX(すべて英数字。英字は小文字のみ)、APPLICATION_NAMEを適宜変更する。
PREFIXは`pwgen -A`で生成することを推奨する。

```shell
$ pwgen -A
```

pwgenがない場合は`brew install pwgen`でインストールしてください。

## ビルド方法

ビルドを行う前に、Docker for Macを必ず起動してください。

ビルドを行う前に以下のコマンドでDBだけを起動します。(sqlxがDBに接続できるようにするため)

```shell
$ makers docker-compose-up-db
```

```shell
$ makers build
```

注意: sqlxにて発行するSQLを追加、修正した際は、各プロジェクト直下で`cargo sqlx prepare`を実行する必要があります(`.sqlx/`にJSONファイルが生成されます)が、`makers build`時に自動的に`cargo sqlx prepare`が実行されるように設定しています。
なので、SQLを追加、修正した際は必ず`makers build`を実行してください。生成された`.sqlx/*.json`ファイルはgitの管理下に含めてください。このときに、DBサーバに接続する必要があるので、`makers docker-compose-up-db`を実行しておく必要があります。

## テスト方法

テストを実行する前に、Docker for Macを必ず起動してください。

```shell
$ makers test
```

注意: `cargo test`でもテスト可能なのですが、今回はtestcontainerを使っている関係で同時に実行するテスト数を環境変数(`RUST_TEST_THREADS=1`)で制限しています。
つまり、`RUST_TEST_THREADS=1 cargo test`でないとテストが正常に実行できないのでご注意ください。
`makers test`ではデフォルトで`RUST_TEST_THREADS=1`となっているので、そのまま実行しても問題ありません。