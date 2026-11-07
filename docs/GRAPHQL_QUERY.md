## GraphQL Query

## How to check

If you want to try out a GraphQL query, open and run the GraphQL IDE as follows

```shell
# with read-api-server running
$ makers open-query-graphql-ide
```

Paste the query in the upper input field, and paste the following in the lower input field (Variables) with reference to
the following, then press the play button.

```json
{
  "groupChatId": "01H7CD0JFZHGYH09AVCP6AD1ZC",
  "userAccountId": "01H7C6DWMK1BKS1JYH1XZE529M"
}
```

## GetGroupChats

Get a summary list of the group chats you have joined.

- userAccountId: browsing account ID

```graphql
query GetGroupChats($userAccountId: String!) {
  groupChats: getGroupChats(userAccountId: $userAccountId) {
    id
    name
  }
}
```

## GetGroupChatNameWithMessages

Get the name of the group chat you are a member of and a list of messages.

- groupChatId: Target group chat ID
- userAccountId: account ID of the browsing account

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

Get a list of members belonging to a group chat in which you are a member.

- groupChatId: ID of the target group chat
- userAccountId: account ID of the browsing account

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