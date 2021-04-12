pub const BUILD_RECORD: &str = r#"
    CREATE TABLE IF NOT EXISTS record (
        pos           BLOB NOT NULL UNIQUE,
        cache         BLOB NOT NULL,
        modified_time BLOB NOT NULL
    )"#;
pub const INSERT_RECORD: &str = r#"
    INSERT INTO record
        (pos, cache, modified_time)
    VALUES
        (?1, ?2, ?3)
    "#;
pub const DELETE_RECORD: &str = r#"
    DELETE FROM
        record
    WHERE
        pos = (?1)
    "#;
pub const FETCH_RECORD: &str = r#"
    SELECT
        *
    FROM
        record
    WHERE
        pos = (?1)
    "#;
pub const FETCH_ALL_POS: &str = r#"
    SELECT
        pos
    FROM
        record
"#;
