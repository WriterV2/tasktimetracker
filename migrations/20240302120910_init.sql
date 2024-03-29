CREATE TABLE IF NOT EXISTS task
(
    id   INTEGER     PRIMARY KEY NOT NULL,
    name VARCHAR(30)             NOT NULL UNIQUE DEFAULT '',
    des  TEXT                    NOT NULL DEFAULT '',
    done BOOLEAN                 NOT NULL DEFAULT 0,
    time INTEGER                 NOT NULL DEFAULT 0,
    iid  INTEGER                 NOT NULL,
    FOREIGN KEY (iid) REFERENCES importance(id)
);

CREATE TABLE IF NOT EXISTS importance
(
    id   INTEGER      PRIMARY KEY NOT NULL,
    name VARCHAR(30)              NOT NULL UNIQUE,
    val  INTEGER                  NOT NULL UNIQUE DEFAULT 0
);

CREATE TABLE IF NOT EXISTS tag
(
    id   INTEGER      PRIMARY KEY NOT NULL,
    name VARCHAR(30)              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS tagassignment
(
    tkid INTEGER                  NOT NULL,
    tgid INTEGER                  NOT NULL,
    FOREIGN KEY (tkid) REFERENCES task(id),
    FOREIGN KEY (tgid) REFERENCES tag(id),
    PRIMARY KEY (tkid, tgid)
);
