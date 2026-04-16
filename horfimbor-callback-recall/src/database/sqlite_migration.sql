CREATE TABLE IF NOT EXISTS callbacks (
                                         id          INTEGER     NOT NULL PRIMARY KEY,
                                         identifier  TEXT        NOT NULL,
                                         payload     BLOB        NOT NULL,
                                         due_date      TIMESTAMPTZ NOT NULL,
                                         status      TEXT        NOT NULL DEFAULT 'pending',
                                         created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                         fired_at    TIMESTAMPTZ,
                                         failed_at   TIMESTAMPTZ,
                                         error_msg   TEXT
);

CREATE INDEX IF NOT EXISTS idx_callbacks_status_due
    ON callbacks (status, due_date);