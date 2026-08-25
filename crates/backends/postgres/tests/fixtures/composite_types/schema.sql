CREATE SCHEMA types;
CREATE ROLE type_owner NOLOGIN;

CREATE TYPE types.postal_address AS (
    street text COLLATE "C",
    city text,
    postal_code varchar(12)
);

ALTER TYPE types.postal_address OWNER TO type_owner;

COMMENT ON TYPE types.postal_address IS
    'Reusable postal address';

COMMENT ON COLUMN types.postal_address.city IS
    'Postal locality';

CREATE TABLE types.contacts (
    address types.postal_address
);
