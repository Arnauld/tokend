\c tokend

CREATE SCHEMA IF NOT EXISTS public;
GRANT ALL ON SCHEMA public TO postgres;
GRANT ALL ON SCHEMA public TO public;
CREATE EXTENSION IF NOT EXISTS hstore SCHEMA public;
CREATE EXTENSION IF NOT EXISTS pg_stat_statements SCHEMA public;

CREATE OR REPLACE PROCEDURE create_role_if_not_exists(role_name text, stmt text)
    LANGUAGE plpgsql AS
$proc$
BEGIN
    IF NOT EXISTS (
            SELECT FROM pg_catalog.pg_roles  -- SELECT list can be empty for this
            WHERE  rolname = role_name) THEN
        EXECUTE format('CREATE ROLE %I %s', role_name, stmt);
    END IF;
END
$proc$;
-- ;;
