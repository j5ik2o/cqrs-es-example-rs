CREATE TABLE `read_model_positions`
(
    `aggregate_id` varchar(64) NOT NULL,
    `seq_nr`       bigint      NOT NULL,
    `updated_at`   datetime    NOT NULL,
    PRIMARY KEY (`aggregate_id`)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8mb4;
