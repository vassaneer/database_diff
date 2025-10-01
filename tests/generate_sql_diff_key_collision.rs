#[cfg(test)]
mod tests {
    use db_diff::{SchemaDiff, ColumnInfo, ColumnDifference, generate_sql_diff};

    #[test]
    fn test_generate_sql_diff_key_collision_documentation() {
        // This test documents the key collision issue in the current implementation
        // where generic keys cause entries to overwrite each other

        let diff = SchemaDiff {
            tables_only_in_first: vec![
                ("public".to_string(), "table1".to_string()),
                ("public".to_string(), "table2".to_string()),
            ],
            tables_only_in_second: vec![
                ("public".to_string(), "table3".to_string()),
                ("public".to_string(), "table4".to_string()),
            ],
            columns_only_in_first: vec![
                ColumnInfo::builder(
                    "public".to_string(),
                    "users".to_string(),
                    "col1".to_string(),
                    "varchar".to_string(),
                    "varchar(255)".to_string(),
                    "YES".to_string(),
                )
                .set_character_maximum_length(255),
                ColumnInfo::builder(
                    "public".to_string(),
                    "users".to_string(),
                    "col2".to_string(),
                    "integer".to_string(),
                    "int".to_string(),
                    "NO".to_string(),
                )
            ],
            columns_only_in_second: vec![
                ColumnInfo::builder(
                    "public".to_string(),
                    "users".to_string(),
                    "col3".to_string(),
                    "timestamp".to_string(),
                    "timestamp".to_string(),
                    "YES".to_string(),
                ),
                ColumnInfo::builder(
                    "public".to_string(),
                    "users".to_string(),
                    "col4".to_string(),
                    "boolean".to_string(),
                    "boolean".to_string(),
                    "YES".to_string(),
                )
            ],
            columns_with_different_definitions: vec![
                ColumnDifference {
                    table_name: "products".to_string(),
                    column_name: "price".to_string(),
                    first: ColumnInfo::builder(
                        "public".to_string(),
                        "products".to_string(),
                        "price".to_string(),
                        "decimal".to_string(),
                        "decimal".to_string(),
                        "NO".to_string(),
                    ),
                    second: ColumnInfo::builder(
                        "public".to_string(),
                        "products".to_string(),
                        "price".to_string(),
                        "decimal".to_string(),
                        "decimal".to_string(),
                        "NO".to_string(),
                    )
                    .set_default("0.00".to_string()),
                }
            ],
        };

        let sql_statements = generate_sql_diff(&diff);

        // Document the key collision issue:
        // 1. For tables_only_in_first, only the last table ("table2") will be in the result
        //    because all use the same key "Drop table in Schema 1 (Schema 1 only)"
        assert!(sql_statements.contains_key("Drop table in Schema 1 (Schema 1 only)"));
        // We can't assert which specific table because of the collision

        // 2. For tables_only_in_second, only the last table ("table4") will be in the result
        //    because all use the same key "Drop table in Schema 2 (Schema 2 only)"
        assert!(sql_statements.contains_key("Drop table in Schema 2 (Schema 2 only)"));
        // We can't assert which specific table because of the collision

        // 3. For columns_only_in_first, only the last column ("col2") will be in the result
        //    because all use the same key "Drop column in Schema 1 (Schema 1 only)"
        assert!(sql_statements.contains_key("Drop column in Schema 1 (Schema 1 only)"));
        // We can't assert which specific column because of the collision

        // 4. For columns_only_in_second, only the last column ("col4") will be in the result
        //    because all use the same key "Drop column in Schema 2 (Schema 2 only)"
        assert!(sql_statements.contains_key("Drop column in Schema 2 (Schema 2 only)"));
        // We can't assert which specific column because of the collision

        // Show that we have fewer entries than we put in due to key collisions

        // Verify we have the expected number of unique keys (10)
        // This is fewer than the total number of differences due to key collisions
        assert_eq!(sql_statements.len(), 10);
    }
}
