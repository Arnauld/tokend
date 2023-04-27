--
--
-- AUDIT
--
--
-- tag::audit_caller_type[]
CREATE TYPE caller_type AS ENUM('USER', 'SERVICE');
-- end::audit_caller_type[]

-- tag::audit_meta[]
CREATE OR REPLACE PROCEDURE add_audit_meta(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('ALTER TABLE %I ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN created_by TEXT                     NOT NULL', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN created_by_type caller_type         NOT NULL', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE NOT NULL', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN updated_by TEXT                     NOT NULL', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN updated_by_type caller_type         NOT NULL', table_name);
END
$proc$;
-- end::audit_meta[]


-- tag::audit_meta_fields[]
CREATE OR REPLACE FUNCTION audit_meta_fields() RETURNS TEXT[]
LANGUAGE plpgsql AS
$proc$
BEGIN
    RETURN '{"created_at", "created_by", "created_by_type", "updated_at", "updated_by", "updated_by_type"}'::TEXT[];
END
$proc$;
-- end::audit_meta_fields[]


-- tag::version_meta_fields[]
CREATE OR REPLACE FUNCTION technical_meta_fields() RETURNS TEXT[]
LANGUAGE plpgsql AS
$proc$
BEGIN
    RETURN audit_meta_fields() || '{"row_version"}'::TEXT[];
END
$proc$;
-- end::version_meta_fields[]

-- tag::audit_deletable[]
CREATE OR REPLACE PROCEDURE add_delete_meta(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('ALTER TABLE %I ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN deleted_by TEXT                    ', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN deleted_by_type caller_type        ', table_name);
END
$proc$;
-- end::audit_deletable[]

-- tag::audit_delete_fields[]
CREATE OR REPLACE FUNCTION audit_delete_fields() RETURNS TEXT[]
LANGUAGE plpgsql AS
$proc$
BEGIN
    RETURN '{"deleted_at", "deleted_by", "deleted_by_type"}'::TEXT[];
END
$proc$;
-- end::audit_delete_fields[]

-- tag::audit_deactivatable[]
CREATE OR REPLACE PROCEDURE add_deactivate_meta(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('ALTER TABLE %I ADD COLUMN deactivated_at TIMESTAMP WITH TIME ZONE', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN deactivated_by TEXT                    ', table_name);
   EXECUTE format('ALTER TABLE %I ADD COLUMN deactivated_by_type caller_type        ', table_name);
END
$proc$;
-- end::audit_deactivatable[]

-- tag::audit_deactivate_fields[]
CREATE OR REPLACE FUNCTION audit_deactivate_fields() RETURNS TEXT[]
LANGUAGE plpgsql AS
$proc$
BEGIN
    RETURN '{"deactivated_at", "deactivated_by", "deactivated_by_type"}'::TEXT[];
END
$proc$;
-- end::audit_deactivate_fields[]


-- tag::audit_meta_trigger[]
CREATE OR REPLACE FUNCTION audit_meta_trigger_func()
RETURNS trigger AS $body$
DECLARE
    caller_id   TEXT;
    caller_type caller_type;
    overrides   BOOLEAN;
BEGIN
    overrides = TRUE;
    IF current_setting('var.bypass_audit_meta', 't') IS NOT NULL THEN
        overrides = FALSE;
    END IF;

    caller_id   = current_setting('var.caller_id'); -- NOSONAR
    caller_type = current_setting('var.caller_type')::caller_type; -- NOSONAR

    IF (TG_OP = 'INSERT') THEN -- NOSONAR
        IF (overrides OR NEW.created_at IS NULL) THEN
            NEW.created_at = now();
        END IF;
        IF (overrides OR NEW.created_by IS NULL) THEN
            NEW.created_by = caller_id;
        END IF;
        IF (overrides OR NEW.created_by_type IS NULL) THEN
            NEW.created_by_type = caller_type;
        END IF;
    END IF;

    IF (overrides OR NEW.updated_at IS NULL) THEN
        NEW.updated_at = now();
    END IF;
    IF (overrides OR NEW.updated_by IS NULL) THEN
        NEW.updated_by = caller_id;
    END IF;
    IF (overrides OR NEW.updated_by_type IS NULL) THEN
        NEW.updated_by_type = caller_type;
    END IF;

    RETURN NEW;
END;
$body$
LANGUAGE plpgsql;

CREATE OR REPLACE PROCEDURE add_audit_meta_trigger(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('CREATE TRIGGER %1$I_audit_meta_trigger'
                  ' BEFORE INSERT OR UPDATE ON %1$I'
                  ' FOR EACH ROW EXECUTE FUNCTION audit_meta_trigger_func()',
                  table_name);
END
$proc$;
-- end::audit_meta_trigger[]

-- tag::set_delete_fields_trigger_func[]
CREATE OR REPLACE FUNCTION set_delete_fields_trigger_func()
RETURNS trigger AS $body$
DECLARE
    caller_id   TEXT;
    caller_type caller_type;
BEGIN
    caller_id   = current_setting('var.caller_id'); -- NOSONAR
    caller_type = current_setting('var.caller_type')::caller_type; -- NOSONAR

    IF (TG_OP = 'UPDATE' AND NEW.deleted_at is NULL) THEN -- NOSONAR
	        NEW.deleted_by = null;
	        NEW.deleted_by_type = null;
    END IF;
    IF (TG_OP = 'UPDATE' AND NEW.deleted_at is NOT NULL) THEN
	        NEW.deleted_by = caller_id;
	        NEW.deleted_by_type = caller_type;
    END IF;

    RETURN NEW;
END;
$body$
LANGUAGE plpgsql;

CREATE OR REPLACE PROCEDURE add_set_delete_fields_trigger(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('CREATE TRIGGER %1$I_set_delete_fields_trigger'
                  ' BEFORE UPDATE ON %1$I' -- NOSONAR
                  ' FOR EACH ROW ' -- NOSONAR
                  ' WHEN (OLD.deleted_at IS DISTINCT FROM NEW.deleted_at) '
                  ' EXECUTE FUNCTION set_delete_fields_trigger_func()',
                  table_name);
END
$proc$;
-- end::set_delete_fields_trigger_func[]

-- tag::set_deactivate_fields_trigger_func[]
CREATE OR REPLACE FUNCTION set_deactivate_fields_trigger_func()
RETURNS trigger AS $body$
DECLARE
    caller_id   TEXT;
    caller_type caller_type;
BEGIN
    caller_id   = current_setting('var.caller_id'); --NOSONAR
    caller_type = current_setting('var.caller_type')::caller_type; -- NOSONAR

    IF (TG_OP = 'UPDATE' AND NEW.deactivated_at is NULL) THEN
	        NEW.deactivated_by = null;
	        NEW.deactivated_by_type = null;
    END IF;
    IF (TG_OP = 'UPDATE' AND NEW.deactivated_at is NOT NULL) THEN
	        NEW.deactivated_by = caller_id;
	        NEW.deactivated_by_type = caller_type;
    END IF;

    RETURN NEW;
END;
$body$
LANGUAGE plpgsql;

CREATE OR REPLACE PROCEDURE add_set_deactivate_fields_trigger(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('CREATE TRIGGER %1$I_set_deactivate_fields_trigger'
                  ' BEFORE UPDATE ON %1$I' -- NOSONAR
                  ' FOR EACH ROW ' -- NOSONAR
                  ' WHEN (OLD.deactivated_at IS DISTINCT FROM NEW.deactivated_at) '
                  ' EXECUTE FUNCTION set_deactivate_fields_trigger_func()',
                  table_name);
END
$proc$;
-- end::set_deactivate_fields_trigger_func[]

-- tag::add_row_version_meta[]
CREATE OR REPLACE PROCEDURE add_row_version_meta(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('ALTER TABLE %I ADD COLUMN row_version INTEGER NOT NULL DEFAULT 1', table_name);
END
$proc$;
-- end::add_row_version_meta[]

-- tag::prevent_from_concurrent_update[]
CREATE OR REPLACE FUNCTION prevent_from_concurrent_update()
RETURNS trigger AS $body$
BEGIN
    IF (NEW.row_version != OLD.row_version + 1) THEN
           RAISE EXCEPTION 'The modification was canceled because of a concurrent update being performed on the same entity (row_version)' USING ERRCODE = '23V01';
    END IF;

   return NEW;
END;
$body$
LANGUAGE plpgsql;
-- end::prevent_from_concurrent_update[]

-- tag::add_prevent_from_concurrent_update_trigger[]
CREATE OR REPLACE PROCEDURE add_prevent_from_concurrent_update_trigger(table_name text)
  LANGUAGE plpgsql AS
$proc$
BEGIN
   EXECUTE format('CREATE TRIGGER %1$I_prevent_from_concurrent_update'
                  ' BEFORE UPDATE ON %1$I' -- NOSONAR
                  ' FOR EACH ROW ' -- NOSONAR
                  ' EXECUTE FUNCTION prevent_from_concurrent_update()',
                  table_name);
END
$proc$;
-- end::add_prevent_from_concurrent_update_trigger[]
