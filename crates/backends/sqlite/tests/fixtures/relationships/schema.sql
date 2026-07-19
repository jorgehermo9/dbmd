CREATE TABLE albums (
    artist TEXT NOT NULL,
    title TEXT NOT NULL,
    PRIMARY KEY (artist, title)
) WITHOUT ROWID;

CREATE TABLE tracks (
    id INTEGER PRIMARY KEY NOT NULL,
    album_artist TEXT NOT NULL,
    album_title TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (album_artist, album_title)
        REFERENCES albums (artist, title)
        ON UPDATE CASCADE
        ON DELETE RESTRICT
);
