# GraphQL Query

## 確認方法

GraphQLのクエリを試したい場合は以下の手順でGraphQL IDEを開いて実行してみてください。

```shell
# read-api-serverを起動した状態で
$ makers open-query-graphql-ide
```

上段の入力欄にクエリを貼り付け、下段の入力欄(Variables)に以下を参考に貼り付け、再生ボタンを押してください。

```json
{
  "groupChatId": "01H7CD0JFZHGYH09AVCP6AD1ZC",
  "userAccountId": "01H7C6DWMK1BKS1JYH1XZE529M"
}
```

## GetGroupChats

参加しているグループチャットのサマリ一覧を取得する

- userAccountId: 閲覧アカウントID

```graphql
query GetGroupChats($userAccountId: String!) {
  groupChats: getGroupChats(userAccountId: $userAccountId) {
    id
    name
  }
}
```

## GetGroupChatNameWithMessages

参加しているグループチャットの名前とメッセージ一覧を取得する

- groupChatId: 対象グループチャットID
- userAccountId: 閲覧アカウントID

```graphql
query GetGroupChatNameWithMessages($groupChatId: String!, $userAccountId: String!) {
  groupChat: getGroupChat(groupChatId: $groupChatId, userAccountId: $userAccountId) {
    name
  }
  messages: getMessages(groupChatId: $groupChatId, userAccountId: $userAccountId) {
    id
    text
    createdAt
  }
}
```

## GetMembers

参加しているグループチャットに所属しているメンバーを一覧で取得できる

- groupChatId: 対象グループチャットID
- userAccountId: 閲覧アカウントID

```graphql
query GetMembers($groupChatId: String!, $userAccountId: String!) {
  members: getMembers(groupChatId: $groupChatId, userAccountId: $userAccountId) {
    id
    userAccountId
    role
    createdAt
  }
}
```

