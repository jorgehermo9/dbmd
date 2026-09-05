CREATE SCHEMA billing;

CREATE TABLE billing.accounts (
    tenant_id bigint NOT NULL,
    account_id bigint NOT NULL,
    email text,
    CONSTRAINT accounts_pk PRIMARY KEY (tenant_id, account_id),
    CONSTRAINT accounts_tenant_email_unique UNIQUE (tenant_id, email)
);

CREATE TABLE billing.invoices (
    tenant_id bigint NOT NULL,
    account_id bigint NOT NULL,
    invoice_number text NOT NULL,
    CONSTRAINT invoices_account_fk
        FOREIGN KEY (tenant_id, account_id)
        REFERENCES billing.accounts (tenant_id, account_id)
        MATCH FULL
        ON UPDATE CASCADE
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED
);
