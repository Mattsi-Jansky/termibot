CREATE TABLE `Entries` (
                           `IdName` VARCHAR(256) NOT NULL,
                           `DisplayName` VARCHAR(256) NOT NULL,
                           `Karma` INT NOT NULL,
                           PRIMARY KEY (`IdName`)
);
CREATE TABLE `Reasons` (
                           `Id` INTEGER(256) NOT NULL,
                           `Name` VARCHAR(256) NOT NULL,
                           `Change` INT NOT NULL,
                           `Value` VARCHAR(1024) NOT NULL,
                           PRIMARY KEY (`Id`)
);
