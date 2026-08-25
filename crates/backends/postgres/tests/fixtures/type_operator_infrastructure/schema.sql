CREATE SCHEMA infrastructure;

CREATE TYPE infrastructure.label_a AS ENUM ('a', 'b');
CREATE TYPE infrastructure.label_b AS ENUM ('a', 'b');
CREATE TYPE infrastructure.label_c;

CREATE FUNCTION infrastructure.label_c_in(cstring)
RETURNS infrastructure.label_c
AS 'int4in'
LANGUAGE internal IMMUTABLE STRICT;

CREATE FUNCTION infrastructure.label_c_out(infrastructure.label_c)
RETURNS cstring
AS 'int4out'
LANGUAGE internal IMMUTABLE STRICT;

CREATE TYPE infrastructure.label_c (
    INPUT = infrastructure.label_c_in,
    OUTPUT = infrastructure.label_c_out,
    INTERNALLENGTH = 4,
    PASSEDBYVALUE,
    ALIGNMENT = int4,
    STORAGE = plain
);

CREATE FUNCTION infrastructure.label_a_to_b(infrastructure.label_a)
RETURNS infrastructure.label_b
LANGUAGE sql
IMMUTABLE
STRICT
RETURN $1::text::infrastructure.label_b;

CREATE CAST (infrastructure.label_a AS infrastructure.label_b)
WITH FUNCTION infrastructure.label_a_to_b(infrastructure.label_a)
AS IMPLICIT;

COMMENT ON CAST (infrastructure.label_a AS infrastructure.label_b) IS
    'Fixture implicit cast';

CREATE CAST (infrastructure.label_b AS infrastructure.label_c)
WITH INOUT AS ASSIGNMENT;

CREATE CAST (infrastructure.label_c AS integer)
WITHOUT FUNCTION;

CREATE DEFAULT CONVERSION infrastructure.utf8_to_latin1
FOR 'UTF8' TO 'LATIN1'
FROM pg_catalog.utf8_to_iso8859_1;

COMMENT ON CONVERSION infrastructure.utf8_to_latin1 IS
    'Fixture default encoding conversion';

CREATE FUNCTION infrastructure.same_integer(integer, integer)
RETURNS boolean
LANGUAGE sql
IMMUTABLE
STRICT
RETURN $1 = $2;

CREATE FUNCTION infrastructure.nonzero(integer)
RETURNS boolean
LANGUAGE sql
IMMUTABLE
STRICT
RETURN $1 <> 0;

CREATE OPERATOR infrastructure.=== (
    FUNCTION = infrastructure.same_integer,
    LEFTARG = integer,
    RIGHTARG = integer,
    COMMUTATOR = OPERATOR(infrastructure.===),
    RESTRICT = pg_catalog.eqsel,
    JOIN = pg_catalog.eqjoinsel,
    HASHES,
    MERGES
);

CREATE OPERATOR infrastructure.!! (
    FUNCTION = infrastructure.nonzero,
    RIGHTARG = integer
);

COMMENT ON OPERATOR infrastructure.=== (integer, integer) IS
    'Fixture equality operator';

CREATE OPERATOR FAMILY infrastructure.integer_family USING btree;

COMMENT ON OPERATOR FAMILY infrastructure.integer_family USING btree IS
    'Fixture integer operator family';

CREATE OPERATOR CLASS infrastructure.integer_class
FOR TYPE integer USING btree
FAMILY infrastructure.integer_family AS
    OPERATOR 1 < (integer, integer),
    OPERATOR 2 <= (integer, integer),
    OPERATOR 3 = (integer, integer),
    OPERATOR 4 >= (integer, integer),
    OPERATOR 5 > (integer, integer),
    FUNCTION 1 pg_catalog.btint4cmp(integer, integer);

COMMENT ON OPERATOR CLASS infrastructure.integer_class USING btree IS
    'Fixture integer operator class';

CREATE FUNCTION infrastructure.fixture_btree_handler(internal)
RETURNS index_am_handler
AS 'bthandler'
LANGUAGE internal
STRICT;

CREATE ACCESS METHOD fixture_btree
TYPE INDEX
HANDLER infrastructure.fixture_btree_handler;

COMMENT ON ACCESS METHOD fixture_btree IS 'Fixture index access method';

CREATE TRUSTED PROCEDURAL LANGUAGE fixture_pl
HANDLER pg_catalog.plpgsql_call_handler
INLINE pg_catalog.plpgsql_inline_handler
VALIDATOR pg_catalog.plpgsql_validator;

COMMENT ON LANGUAGE fixture_pl IS 'Fixture procedural language';

CREATE TRANSFORM FOR integer LANGUAGE fixture_pl (
    FROM SQL WITH FUNCTION pg_catalog.textlike_support(internal),
    TO SQL WITH FUNCTION pg_catalog.int4recv(internal)
);

COMMENT ON TRANSFORM FOR integer LANGUAGE fixture_pl IS
    'Fixture integer transform';

CREATE PROCEDURE infrastructure.accept_integer(value integer)
LANGUAGE fixture_pl
TRANSFORM FOR TYPE integer
AS $$
BEGIN
    NULL;
END;
$$;
