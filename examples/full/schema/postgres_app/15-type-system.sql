CREATE SCHEMA type_system;

CREATE TYPE type_system.pending_value;
COMMENT ON TYPE type_system.pending_value IS 'Forward-declared shell type';

CREATE TYPE type_system.scalar_token;

CREATE FUNCTION type_system.scalar_token_in(cstring)
RETURNS type_system.scalar_token
AS 'int4in'
LANGUAGE internal
IMMUTABLE
STRICT;

CREATE FUNCTION type_system.scalar_token_out(type_system.scalar_token)
RETURNS cstring
AS 'int4out'
LANGUAGE internal
IMMUTABLE
STRICT;

CREATE TYPE type_system.scalar_token (
    INPUT = type_system.scalar_token_in,
    OUTPUT = type_system.scalar_token_out,
    INTERNALLENGTH = 4,
    PASSEDBYVALUE,
    ALIGNMENT = int4,
    STORAGE = plain,
    CATEGORY = 'N',
    PREFERRED = true,
    DEFAULT = '0'
);

COMMENT ON TYPE type_system.scalar_token IS 'Integer-backed application token';

CREATE TYPE type_system.measurement_range AS RANGE (
    SUBTYPE = double precision,
    SUBTYPE_OPCLASS = float8_ops,
    SUBTYPE_DIFF = float8mi,
    MULTIRANGE_TYPE_NAME = type_system.measurement_ranges
);

COMMENT ON TYPE type_system.measurement_range IS 'One continuous measurement interval';
COMMENT ON TYPE type_system.measurement_ranges IS 'Disjoint measurement intervals';

CREATE TABLE type_system.measurements (
    token type_system.scalar_token NOT NULL,
    accepted type_system.measurement_range NOT NULL,
    historical type_system.measurement_ranges NOT NULL
);
