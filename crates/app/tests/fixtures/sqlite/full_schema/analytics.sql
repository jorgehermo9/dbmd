CREATE TABLE daily_metrics (
    metric_date TEXT PRIMARY KEY,
    active_accounts INTEGER NOT NULL CHECK (active_accounts >= 0)
) WITHOUT ROWID, STRICT;

CREATE VIEW latest_metric AS
SELECT metric_date, active_accounts FROM daily_metrics ORDER BY metric_date DESC LIMIT 1;
