-- Add migration script here

CREATE TABLE `group_chats`
(
    `id`         varchar(64) NOT NULL,
    `name`       varchar(64) NOT NULL,
    `owner_id`   varchar(64) NOT NULL,
    `created_at` datetime    NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;

CREATE TABLE `members`
(
    `id`            varchar(64) NOT NULL,
    `group_chat_id` varchar(64) NOT NULL,
    `account_id`    varchar(64) NOT NULL,
    `role`          varchar(64) NOT NULL,
    `created_at`    datetime    NOT NULL,
    PRIMARY KEY (`id`),
    FOREIGN KEY (`group_chat_id`) REFERENCES group_chats (`id`),
    UNIQUE KEY `group_chat_id_account_id` (`group_chat_id`, `account_id`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;

CREATE TABLE `messages`
(
    `id`            varchar(64) NOT NULL,
    `group_chat_id` varchar(64) NOT NULL,
    `account_id`    varchar(64) NOT NULL,
    `text`          TEXT        NOT NULL,
    `created_at`    datetime    NOT NULL,
    PRIMARY KEY (`id`),
    FOREIGN KEY (`group_chat_id`) REFERENCES group_chats (`id`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;
