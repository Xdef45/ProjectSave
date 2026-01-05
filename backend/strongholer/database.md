```
CREATE DATABASE strongholder;
USE strongholder;
CREATE TABLE Creadentials(
    username VARCHAR(255) PRIMARY KEY UNIQUE NOT NULL,
    encrypt_master_key_2 VARCHAR(255) UNIQUE NOT NULL
    );
