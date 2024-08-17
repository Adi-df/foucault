CREATE TABLE notes_table (
    id integer PRIMARY KEY AUTOINCREMENT,
    name string UNIQUE,
    content text
);
