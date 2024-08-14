CREATE TABLE users (
    id BLOB PRIMARY KEY NOT NULL
) WITHOUT ROWID,
STRICT;

CREATE TABLE identitys_users (
    id BLOB PRIMARY KEY NOT NULL,
    user_id BLOB NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
) WITHOUT ROWID,
STRICT;
