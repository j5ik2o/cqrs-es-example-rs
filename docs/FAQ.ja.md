# FAQ

## API仕様を確認したい

[API_SPEC.md](API_SPEC.md)を参照してください。

## 動作確認したい

動作確認方法は2種類あります。

- [docker-composeから起動](./DEBUG_ON_DOCKER_COMPOSE.md)
- [IntelliJから起動](./DEBUG_ON_LOCAL_MACHINE.md)

## データベースの中身を確認したい

1. `makers docker-compose-up` や `makers docker-compose-up-db` でデータベースを起動する
1. DynamoDBの場合は http://localhost:8003/ を開く
1. Auroraの場合は http://localhost:4040/ を開く

## cargo-make(makers)のタスク一覧を知りたい

```shell
$ makers --list-all-steps
```