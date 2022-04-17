CREATE TABLE `guilds` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `name` varchar(100) DEFAULT NULL,
  `prefix` varchar(2) DEFAULT '!',
  `owner_id` bigint(20) DEFAULT NULL,
  `cur_vc_id` varchar(100) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `fk_guilds_owner` (`owner_id`),
  CONSTRAINT `fk_guilds_owner` FOREIGN KEY (`owner_id`) REFERENCES `users` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=902442635606786050 DEFAULT CHARSET=utf8mb4