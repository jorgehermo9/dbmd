CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE VIEW document_summaries (document_id, heading) AS
SELECT id, title FROM documents;

CREATE VIEW document_titles AS
SELECT id, title, length(body) AS body_length FROM documents;

CREATE TRIGGER documents_touch_updated_at
AFTER UPDATE OF title, body ON documents
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE documents
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER document_summaries_insert
INSTEAD OF INSERT ON document_summaries
BEGIN
    INSERT INTO documents (id, title, body)
    VALUES (NEW.document_id, NEW.heading, '');
END;

CREATE TRIGGER documents_prevent_root_delete
BEFORE DELETE ON documents
WHEN OLD.id = 0
BEGIN
    SELECT RAISE(IGNORE);
END;

CREATE VIRTUAL TABLE document_search USING fts5(
    title,
    body,
    content='documents',
    content_rowid='id',
    tokenize='porter unicode61'
);
