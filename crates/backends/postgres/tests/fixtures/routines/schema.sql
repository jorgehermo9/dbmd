CREATE SCHEMA routines;

CREATE FUNCTION routines.starts_with(value text, prefix text DEFAULT '')
RETURNS boolean
LANGUAGE internal
IMMUTABLE
PARALLEL SAFE
STRICT
LEAKPROOF
SUPPORT pg_catalog.text_starts_with_support
COST 3
AS 'text_starts_with';

COMMENT ON FUNCTION routines.starts_with(text, text)
IS 'Planner-supported strict and leakproof function';

CREATE FUNCTION routines.range_values(first_value integer, last_value integer)
RETURNS SETOF integer
LANGUAGE sql
STABLE
PARALLEL RESTRICTED
COST 7
ROWS 25
SET search_path = pg_catalog
AS $$ SELECT generate_series(first_value, last_value) $$;

CREATE FUNCTION routines.row_number_clone()
RETURNS bigint
LANGUAGE internal
WINDOW
IMMUTABLE
PARALLEL SAFE
AS 'window_row_number';
