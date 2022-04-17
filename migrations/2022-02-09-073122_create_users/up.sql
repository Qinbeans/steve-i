CREATE TABLE `users` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `name` varchar(100) DEFAULT NULL,
  `tag` varchar(100) DEFAULT NULL,
  `current_channel_id` longtext DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=931630866936000563 DEFAULT CHARSET=utf8mb4