-- Add migration script here

CREATE TABLE `threads`
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
    `id`         varchar(64) NOT NULL,
    `thread_id`  varchar(64) NOT NULL,
    `account_id` varchar(64) NOT NULL,
    `created_at` datetime    NOT NULL,
    PRIMARY KEY (`id`),
    FOREIGN KEY (`thread_id`) REFERENCES threads (`id`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;

CREATE TABLE `messages`
(
    `id`         varchar(64) NOT NULL,
    `thread_id`  varchar(64) NOT NULL,
    `account_id` varchar(64) NOT NULL,
    `text`       TEXT        NOT NULL,
    `created_at` datetime    NOT NULL,
    PRIMARY KEY (`id`),
    FOREIGN KEY (`thread_id`) REFERENCES threads (`id`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;
