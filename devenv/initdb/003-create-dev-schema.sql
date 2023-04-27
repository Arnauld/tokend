\c tokend

CREATE SCHEMA IF NOT EXISTS dev;

--ROLE tokend_dev
CALL create_role_if_not_exists('tokend_dev', 'WITH PASSWORD ''dev_p'' NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION BYPASSRLS');
ALTER SCHEMA dev OWNER TO tokend_dev;
ALTER ROLE tokend_dev IN DATABASE tokend SET search_path to dev, public;

--ROLE tokend_dev_APP
CALL create_role_if_not_exists('tokend_dev_app', 'WITH PASSWORD ''dev_p'' NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB LOGIN NOREPLICATION NOBYPASSRLS');
ALTER ROLE tokend_dev_app IN DATABASE tokend SET search_path to dev, public;
GRANT SELECT, UPDATE, INSERT, DELETE ON ALL TABLES    IN SCHEMA dev TO tokend_dev_app;
GRANT SELECT, UPDATE, USAGE          ON ALL SEQUENCES IN SCHEMA dev TO tokend_dev_app;
GRANT USAGE                          ON                  SCHEMA dev TO tokend_dev_app;
-- limit privileges that will be applied to objects created in the future, e.g. new tables
ALTER DEFAULT PRIVILEGES FOR ROLE tokend_dev IN SCHEMA dev GRANT SELECT, UPDATE, INSERT, DELETE ON TABLES    TO tokend_dev_app;
ALTER DEFAULT PRIVILEGES FOR ROLE tokend_dev IN SCHEMA dev GRANT SELECT, UPDATE, USAGE          ON SEQUENCES TO tokend_dev_app;

--ROLE tokend_dev_MIG
CALL create_role_if_not_exists('tokend_dev_mig', 'WITH PASSWORD ''dev_p'' NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB LOGIN NOREPLICATION BYPASSRLS');
ALTER ROLE tokend_dev_mig IN DATABASE tokend SET search_path to dev, public;
GRANT tokend_dev TO tokend_dev_mig;

--ROLE dev_READONLY
CALL create_role_if_not_exists('tokend_dev_readonly', 'WITH PASSWORD ''dev_p'' NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB LOGIN NOREPLICATION NOBYPASSRLS');
ALTER ROLE tokend_dev_readonly IN DATABASE tokend SET search_path to dev, public;
GRANT SELECT ON ALL TABLES IN SCHEMA dev TO tokend_dev_readonly;
GRANT USAGE  ON               SCHEMA dev TO tokend_dev_readonly;
