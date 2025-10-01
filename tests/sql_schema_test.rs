#[cfg(test)]
mod tests {
    use db_diff::{ColumnInfo, build_schema_map, compare_schema_maps, generate_sql_diff};

    #[test]
    fn test_sql_generation_includes_schema_names() {
        // Create test data with schema information
        let columns1 = vec![
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "name".to_string(),
                "varchar".to_string(),
                "varchar(255)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(255),
        ];

        let columns2 = vec![
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "email".to_string(),
                "varchar".to_string(),
                "varchar(255)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(255)
            .set_column_comment("test".to_string()),
        ];

        let map1 = build_schema_map(columns1);
        let map2 = build_schema_map(columns2);

        let diff = compare_schema_maps(&map1, &map2);

        let sql_statements = generate_sql_diff(&diff);

        // Verify that ALTER TABLE statements include schema names
        let has_drop_with_schema = sql_statements.values().any(|s| s.contains("ALTER TABLE `public`.users DROP COLUMN name"));
        let has_add_with_schema = sql_statements.values().any(|s| s.contains("ALTER TABLE `public`.users ADD COLUMN email varchar(255) NULL COMMENT 'test';"));

        assert!(has_drop_with_schema, "Should have ALTER TABLE statement with schema name for DROP COLUMN");
        assert!(has_add_with_schema, "Should have ALTER TABLE statement with schema name for ADD COLUMN");
    }
}
