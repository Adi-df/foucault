CREATE TABLE tags_table (
    id integer PRIMARY KEY AUTOINCREMENT,
    name string UNIQUE NOT NULL,
    color integer NOT NULL
);
