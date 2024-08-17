CREATE TABLE links_table (
    id integer PRIMARY KEY AUTOINCREMENT,
    from_id integer NOT NULL,
    to_name text NOT NULL,
    FOREIGN KEY (from_id) REFERENCES notes_table (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);
