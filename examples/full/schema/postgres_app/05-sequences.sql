CREATE SCHEMA automation;

CREATE TABLE automation.invoices (
    id bigint NOT NULL
);

CREATE UNLOGGED SEQUENCE automation.invoice_number
    AS bigint
    INCREMENT BY 5
    MINVALUE 1000
    MAXVALUE 999999
    START WITH 1000
    CACHE 20
    CYCLE;

ALTER SEQUENCE automation.invoice_number
    OWNED BY automation.invoices.id;

ALTER TABLE automation.invoices
    ALTER COLUMN id SET DEFAULT nextval('automation.invoice_number');

COMMENT ON SEQUENCE automation.invoice_number IS
    'Stable invoice number allocator';
