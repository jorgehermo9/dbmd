CREATE TABLE accounts (
    account_id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
    email VARCHAR(255) NOT NULL
) ENGINE=InnoDB;

CREATE PROCEDURE rotate_accounts()
READS SQL DATA
SELECT COUNT(*) FROM accounts;

CREATE SERVER dbmd_remote
FOREIGN DATA WRAPPER mysql
OPTIONS (
    USER 'remote_user',
    PASSWORD 'dbmd-server-secret',
    HOST '198.51.100.10',
    DATABASE 'remote_database',
    PORT 3306,
    SOCKET '',
    OWNER 'dbmd'
);

CREATE SPATIAL REFERENCE SYSTEM 33001
NAME 'dbmd geographic'
ORGANIZATION 'dbmd' IDENTIFIED BY 33001
DEFINITION 'GEOGCS["dbmd geographic",DATUM["dbmd datum",SPHEROID["dbmd sphere",6378137,298.257223563]],PRIMEM["Greenwich",0],UNIT["degree",0.017453292519943278],AXIS["Latitude",NORTH],AXIS["Longitude",EAST]]'
DESCRIPTION 'Application-defined fixture SRS';

CREATE TABLESPACE dbmd_general
ADD DATAFILE 'dbmd-general.ibd'
AUTOEXTEND_SIZE = 4M
ENGINE=InnoDB;

CREATE RESOURCE GROUP dbmd_batch
TYPE = USER
THREAD_PRIORITY = 0
DISABLE;

CREATE ROLE 'dbmd_reader'@'%';

CREATE USER 'dbmd_app'@'%'
IDENTIFIED BY 'dbmd-account-secret'
REQUIRE SSL
WITH MAX_QUERIES_PER_HOUR 10 MAX_USER_CONNECTIONS 2
PASSWORD EXPIRE INTERVAL 90 DAY
ACCOUNT LOCK
COMMENT 'dbmd application account';

ALTER USER 'dbmd_app'@'%' ATTRIBUTE '{"team":"platform"}';

GRANT SELECT ON test.* TO 'dbmd_reader'@'%';
GRANT UPDATE (email) ON test.accounts TO 'dbmd_reader'@'%';
GRANT EXECUTE ON PROCEDURE test.rotate_accounts TO 'dbmd_reader'@'%';
GRANT PROXY ON 'dbmd_reader'@'%' TO 'dbmd_app'@'%' WITH GRANT OPTION;
GRANT 'dbmd_reader'@'%' TO 'dbmd_app'@'%' WITH ADMIN OPTION;
SET DEFAULT ROLE 'dbmd_reader'@'%' TO 'dbmd_app'@'%';

INSTALL PLUGIN auth_socket SONAME 'auth_socket.so';
INSTALL COMPONENT 'file://component_validate_password';

ALTER DATABASE test READ ONLY = 1;
