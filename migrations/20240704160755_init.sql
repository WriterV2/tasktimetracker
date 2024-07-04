CREATE TABLE IF NOT EXISTS booking
(
    id        INTEGER     PRIMARY KEY NOT NULL,
    startdate INTEGER                 NOT NULL,
    enddate   INTEGER,
    des       TEXT                    NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS tag
(
    id   INTEGER      PRIMARY KEY NOT NULL,
    name VARCHAR(30)              NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS tagassignment
(
    bid INTEGER                   NOT NULL,
    tgid INTEGER                  NOT NULL,
    FOREIGN KEY (bid)  REFERENCES booking(id),
    FOREIGN KEY (tgid) REFERENCES tag(id),
    PRIMARY KEY (bid, tgid)
);
