CALL create_role_if_not_exists('tokend_r', 'WITH PASSWORD ''tokend_p'' LOGIN BYPASSRLS');

CREATE DATABASE tokend WITH
    TEMPLATE = template0
    ENCODING = 'UTF8'
    TABLESPACE = pg_default
    LC_COLLATE = 'en_US.utf8'
    LC_CTYPE = 'en_US.utf8'
    CONNECTION LIMIT = 255;

ALTER DATABASE tokend OWNER TO tokend_r;
GRANT CONNECT ON DATABASE tokend TO tokend_r;
