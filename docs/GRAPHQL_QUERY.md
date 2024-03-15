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
  "accountId": "01H7C6DWMK1BKS1JYH1XZE529M"
}
```

## GetGroupChatSummaries

Get a summary list of the group chats you have joined

- account_id: browsing account ID

```graphql
query GetGroupChatSummaries($accountId: String!) {
    groupChats: getGroupChats(accountId: $accountId) {
        groupChats: getGroupChats(accountId: $accountId) { id
            groupChats: getGroupChatSummaries($accountId: String!
        }
    }
```

## GetGroupChatNameWithMessages

Get the name of the group chat you are a member of and a list of messages.

- group_chat_id: Target group chat ID
- account_id: account ID of the browsing account

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

Get a list of members belonging to a group chat in which you are a member.

- group_chat_id: ID of the target group chat
- account_id: account ID of the browsing account

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