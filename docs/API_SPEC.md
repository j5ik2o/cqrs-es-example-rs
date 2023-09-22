# API仕様の確認方法

## Write API Server: OpenAPIの仕様を確認する方法

```shell
# CLIで確認する場合
$ makers view-openapi
# ファイルに出力する場合
$ makers export-openapi ./openapi.yaml
```

### Swagger UIで確認する場合

まずはWrite API Serverを起動する

```shell
$ makers docker-compose-up-db
$ makers run-write-api-server
```

もしくは

```shell
$ makers docker-build-local-all # ビルドに時間が掛かります
$ makers docker-compose-up
```

以下のコマンドでswagger-uiをブラウザで開きます。

```shell
$ makers view-swagger-ui
```

### OpenAPI定義からTypeScriptクライアントを生成する

参照: [frontend/README.md](../../frontend/README.md#openapi-の-typescirpt-クライアントを自動生成する)

## Read API Server: GraphQLのSDLを確認する方法

```shell
# CLIで確認する場合
$ makers view-sdl
# ファイルに出力する場合
$ makers export-sdl ./schema.sdl
```

### SDLからTypeScriptクライアントを生成する

参照 [frontend/README.md](../../frontend/README.md#graphql-の-typescirpt-クライアントを自動生成する)
