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
  "accountId": "01H7C6DWMK1BKS1JYH1XZE529M"
}
```

## GetGroupChatSummaries

参加しているグループチャットのサマリ一覧を取得する

- account_id: 閲覧アカウントID

```graphql
query GetGroupChatSummaries($accountId: String!) {
  groupChats: getGroupChats(accountId: $accountId) {
    id
    name
  }
}
```

## GetGroupChatNameWithMessages

参加しているグループチャットの名前とメッセージ一覧を取得する

- group_chat_id: 対象グループチャットID
- account_id: 閲覧アカウントID

```graphql
query GetGroupChatNameWithMessages($groupChatId: String!, $accountId: String!) {
  groupChat: getGroupChat(groupChatId: $groupChatId, accountId: $accountId) {
    name
  }
  messages: getMessages(groupChatId: $groupChatId, accountId: $accountId) {
    id
    text
    createdAt
  }
}
```

## GetMembers

参加しているグループチャットに所属しているメンバーを一覧で取得できる

- group_chat_id: 対象グループチャットID
- account_id: 閲覧アカウントID

```graphql
query GetMembers($groupChatId: String!, $accountId: String!) {
  members: getMembers(groupChatId: $groupChatId, accountId: $accountId) {
    id
    accountId
    role
    createdAt
  }
}
```

