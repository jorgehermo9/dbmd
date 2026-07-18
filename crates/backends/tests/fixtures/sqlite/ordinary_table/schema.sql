CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT DEFAULT 'anonymous'
);
