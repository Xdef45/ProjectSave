```
CREATE DATABASE strongholder;
USE strongholder;
CREATE TABLE Credentials(
    id VARCHAR(255) PRIMARY KEY UNIQUE NOT NULL,
    username VARCHAR(255) UNIQUE NOT NULL,
    encrypt_master_key_2 VARCHAR(80) UNIQUE NOT NULL
    );
```
