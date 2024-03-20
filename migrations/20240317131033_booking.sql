ALTER TABLE task
DROP COLUMN time;

CREATE TABLE IF NOT EXISTS booking
(
    id        INTEGER     PRIMARY KEY NOT NULL,
    tid       INTEGER                 NOT NULL,
    startdate INTEGER                 NOT NULL,
    enddate   INTEGER
);
