-- Your SQL goes here
CREATE TABLE `guild_users` (
  `user_id` bigint(20) NOT NULL,
  `guild_id` bigint(20) NOT NULL,
  PRIMARY KEY (`user_id`,`guild_id`),
  KEY `fk_guild_users_guild` (`guild_id`),
  CONSTRAINT `fk_guild_users_guild` FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`),
  CONSTRAINT `fk_guild_users_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 