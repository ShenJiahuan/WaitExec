DROP TABLE IF EXISTS `process_list`;
CREATE TABLE `process_list`
(
    `id`       int(11) NOT NULL AUTO_INCREMENT,
    `hostname` varchar(64) DEFAULT NULL,
    `pid`      int(11) DEFAULT NULL,
    `status`   int(11) DEFAULT NULL,
    PRIMARY KEY (`id`)
);