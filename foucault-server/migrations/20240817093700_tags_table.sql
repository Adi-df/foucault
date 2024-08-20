CREATE TABLE tags_table (
    id integer PRIMARY KEY AUTOINCREMENT,
    name text UNIQUE NOT NULL,
    color integer NOT NULL
);
