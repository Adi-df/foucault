CREATE TABLE tags_join_table (
    id integer PRIMARY KEY AUTOINCREMENT,
    note_id integer NOT NULL,
    tag_id integer NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes_table (id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags_table (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);
