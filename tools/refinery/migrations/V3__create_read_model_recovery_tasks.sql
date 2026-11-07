CREATE TABLE `read_model_recovery_tasks`
(
    `store_id`           varchar(64) NOT NULL,
    `projection_name`    varchar(64) NOT NULL,
    `projection_version` varchar(32) NOT NULL,
    `aggregate_id`       varchar(64) NOT NULL,
    `seq_nr`             bigint      NOT NULL,
    `failure_type`       varchar(32) NOT NULL,
    `error_message`      text        NOT NULL,
    `status`             varchar(16) NOT NULL,
    `created_at`         datetime    NOT NULL,
    `updated_at`         datetime    NOT NULL,
    PRIMARY KEY (`store_id`, `projection_name`, `aggregate_id`, `seq_nr`),
    INDEX `idx_read_model_recovery_tasks_status` (`status`, `created_at`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;
