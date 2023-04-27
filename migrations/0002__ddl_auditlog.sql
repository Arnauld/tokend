-- ==================================================================
--
-- AUDIT LOG
--
-- ==================================================================
-- tag:hstore[]
CREATE EXTENSION IF NOT EXISTS hstore;
-- end:hstore[]

-- tag:audit_log_table[]
CREATE TYPE audit_log_category_type AS ENUM ('tenant', 'policy');

CREATE SEQUENCE audit_log_id_seq;

CREATE TABLE audit_log (
                           id                 BIGINT NOT NULL DEFAULT nextval('audit_log_id_seq') PRIMARY KEY,
                           transaction_id     BIGINT,
                           category           audit_log_category_type,
    -- WHO
                           tenant_id          BIGINT REFERENCES tenants(id), -- tenant is declared to keep known order for trigger
                           changed_by         TEXT NOT NULL,
                           changed_by_type    caller_type NOT NULL,
    -- CHANGES
                           changed_at         TIMESTAMP WITH TIME ZONE NOT NULL,
                           changed_table_name TEXT NOT NULL,
                           changed_id         BIGINT NOT NULL, -- changed primary key
                           changed_type       TEXT NOT NULL CHECK (changed_type IN ('I','D','U','T')),
                           changed_fields     JSONB
);
-- end:audit_log_table[]

-- tag:audit_log_trigger[]
CREATE OR REPLACE FUNCTION audit_log_trigger_func()
    RETURNS trigger AS $body$
DECLARE
    category        audit_log_category_type;
    excluded_cols   text[] = ARRAY[]::text[];
    audit_row       audit_log;
    changed_fields  HSTORE;
    before_fields   JSONB;
    after_fields    JSONB;
    diff            JSONB;
    tenant_id       BIGINT;
BEGIN
    category = TG_ARGV[0]::audit_log_category_type;
    IF TG_ARGV[1] IS NOT NULL THEN
        excluded_cols = TG_ARGV[1]::text[];
    END IF;

    -- special case
    IF (TG_TABLE_NAME != 'tenants') THEN -- NOSONAR
        tenant_id = get_current_tenant_id();
    ELSE
        tenant_id = NEW.id;
    END IF;

    audit_row = ROW(
        nextval('audit_log_id_seq'),        -- id
        txid_current(),                     -- transaction_id
        category,
        -- CallingContext...
        tenant_id,                           -- tenant_id
        current_setting('var.caller_id'),    -- caller_id NOSONAR
        current_setting('var.caller_type')::caller_type, -- caller_id_type SONONAR
        --
                CURRENT_TIMESTAMP,                  -- changed_at
        TG_TABLE_NAME,                      -- changed_table_name
        NEW.id,                             -- changed_id
        substring(TG_OP,1,1),               -- changed_type
        NULL                                -- changed_fields
        );

    IF (TG_OP = 'UPDATE') THEN
        -- removes all matching key/value pairs from the 1st hstore that appear in the 2nd hstore
        -- removes the key/value pairs where the keys are found in the array of strings
        -- then convert the hstore to jsonb
        changed_fields =  (hstore(NEW.*) - hstore(OLD.*)) - excluded_cols;
        IF changed_fields = hstore('') THEN
            -- All changed fields are ignored. Skip this update.
            RETURN NEW;
        END IF;
        after_fields = hstore_to_json(changed_fields);
        changed_fields = (hstore(OLD.*) - hstore(NEW.*)) - excluded_cols;
        before_fields = hstore_to_json(changed_fields);

    ELSIF (TG_OP = 'DELETE' AND TG_LEVEL = 'ROW') THEN
        before_fields = hstore_to_json(hstore(OLD.*) - excluded_cols);
        after_fields  = '{}';
        audit_row.changed_id = OLD.id;
    ELSIF (TG_OP = 'INSERT' AND TG_LEVEL = 'ROW') THEN
        before_fields = '{}';
        after_fields = hstore_to_json(hstore(NEW.*) - excluded_cols);
    END IF;

    diff = jsonb_set('{}', '{"before"}', before_fields);
    diff = jsonb_set(diff, '{"after"}', after_fields);
    audit_row.changed_fields = diff;

    INSERT INTO audit_log VALUES (audit_row.*);
    RETURN NEW;
END;
$body$
    LANGUAGE plpgsql;

CREATE OR REPLACE PROCEDURE add_audit_log_trigger(table_name text, category audit_log_category_type, excluded_cols text[] default ARRAY[]::text[])
    LANGUAGE plpgsql AS
$proc$
BEGIN
    EXECUTE format('CREATE TRIGGER %1$I_audit_trigger'
                       ' AFTER INSERT OR UPDATE OR DELETE ON %1$I'
                       ' FOR EACH ROW EXECUTE FUNCTION audit_log_trigger_func(%2$I, %3$L)',
                   table_name, category, excluded_cols);
END
$proc$;
-- end:audit_log_trigger[]


-- tag::tenant_post_creation[]
CALL add_audit_meta('tenants');
CALL add_audit_meta_trigger('tenants');
CALL add_audit_log_trigger('tenants', 'tenant', audit_meta_fields());
-- end::tenant_post_creation[]
