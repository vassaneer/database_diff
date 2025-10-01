#[cfg(test)]
mod tests {
    use db_diff::{ColumnInfo, build_schema_map, compare_schema_maps, generate_sql_diff};

    #[test]
    fn test_drop_table_includes_schema_name() {
        // Create test data with different tables in the same schema
        let columns1 = vec![
            ColumnInfo::builder(
                "public".to_string(),
                "old_users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),
        ];

        let columns2 = vec![
            ColumnInfo::builder(
                "public".to_string(),
                "new_users".to_string(),
                "id".to_string(),
                "bigint".to_string(),
                "bigint".to_string(),
                "NO".to_string(),
            ),
        ];

        let map1 = build_schema_map(columns1);
        let map2 = build_schema_map(columns2);

        let diff = compare_schema_maps(&map1, &map2);
        let sql_statements = generate_sql_diff(&diff);

        // Print the SQL statements for verification
        println!("Generated SQL statements:");
        for (key, statement) in &sql_statements {
            println!("{}: {}", key, statement);
        }

        // Verify that DROP TABLE statements include schema names
        let has_drop_old_table = sql_statements.values().any(|s| s.contains("DROP TABLE `public`.old_users"));
        let has_create_new_table = sql_statements.values().any(|s| s.contains("-- CREATE TABLE `public`.new_users (id bigint NOT NULL,)"));

        assert!(has_drop_old_table, "Should have DROP TABLE statement with schema name");
        assert!(has_create_new_table, "Should have CREATE TABLE statement with schema name");
    }
}
