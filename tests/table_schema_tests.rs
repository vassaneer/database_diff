#[cfg(test)]
mod tests {
    use db_diff::{ColumnInfo, build_schema_map, compare_schema_maps, generate_sql_diff};

    #[test]
    fn test_schema_comparison_with_table_schema() {
        // Create test data with different schemas
        let columns1 = vec![
        ColumnInfo::builder(
            "schema1".to_string(),
            "users".to_string(),
            "name1".to_string(),
            "varchar".to_string(),
            "varchar(255)".to_string(),
            "YES".to_string(),
        )
        .set_character_maximum_length(255),
            ColumnInfo::builder(
                "schema1".to_string(),
                "users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),

        ];
        let columns2 = vec![
            ColumnInfo::builder(
                "schema2".to_string(),
                "users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),
            ColumnInfo::builder(
                "schema2".to_string(),
                "users".to_string(),
                "email".to_string(),
                "varchar".to_string(),
                "varchar(255)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(255),
        ];

        let map1 = build_schema_map(columns1);
        let map2 = build_schema_map(columns2);

        let diff = compare_schema_maps(&map1, &map2);

        // Since the tables are in different schemas, they should be treated as different tables
        // So we should have tables only in first and tables only in second
        assert_eq!(diff.tables_only_in_first.len(), 1);
        assert_eq!(diff.tables_only_in_second.len(), 1);

        // And no column differences since they're considered different tables
        assert_eq!(diff.columns_only_in_first.len(), 0);
        assert_eq!(diff.columns_only_in_second.len(), 0);
        assert_eq!(diff.columns_with_different_definitions.len(), 0);
    }

    #[test]
    fn test_schema_comparison_same_table_different_schemas() {
        // Test that tables with the same name but different schemas are treated as different
        let columns1 = vec![
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),
        ];

        let columns2 = vec![
            ColumnInfo::builder(
                "private".to_string(),
                "users".to_string(),
                "id".to_string(),
                "integer".to_string(),
                "int(11)".to_string(),
                "NO".to_string(),
            ),
        ];

        let map1 = build_schema_map(columns1);
        let map2 = build_schema_map(columns2);

        let diff = compare_schema_maps(&map1, &map2);

        // Tables with same name but different schemas should be treated as different tables
        assert_eq!(diff.tables_only_in_first.len(), 1);
        assert_eq!(diff.tables_only_in_second.len(), 1);
        assert_eq!(diff.columns_only_in_first.len(), 0);
        assert_eq!(diff.columns_only_in_second.len(), 0);
        assert_eq!(diff.columns_with_different_definitions.len(), 0);
    }

    #[test]
    fn test_schema_comparison_same_table_same_schema() {
        // Test that tables with the same name and same schema are compared properly
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
                "bigint".to_string(), // Different data type
                "bigint".to_string(),
                "NO".to_string(),
            ),
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "email".to_string(), // Different column
                "varchar".to_string(),
                "varchar(255)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(255),
        ];

        let map1 = build_schema_map(columns1);
        let map2 = build_schema_map(columns2);

        let diff = compare_schema_maps(&map1, &map2);

        // Should have no tables only in first or second since they're the same table
        assert_eq!(diff.tables_only_in_first.len(), 0);
        assert_eq!(diff.tables_only_in_second.len(), 0);

        // Should have columns only in first (name) and only in second (email)
        assert_eq!(diff.columns_only_in_first.len(), 1);
        assert_eq!(diff.columns_only_in_second.len(), 1);
        assert_eq!(diff.columns_only_in_first[0].column_name, "name");
        assert_eq!(diff.columns_only_in_second[0].column_name, "email");

        // Should have one column with different definitions (id)
        assert_eq!(diff.columns_with_different_definitions.len(), 1);
        assert_eq!(diff.columns_with_different_definitions[0].column_name, "id");
        assert_eq!(diff.columns_with_different_definitions[0].first.data_type, "integer");
        assert_eq!(diff.columns_with_different_definitions[0].second.data_type, "bigint");
    }

    #[test]
    fn test_sql_generation_with_table_schema() {
        // Test that SQL generation includes schema names
        let columns1 = vec![
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
                "email".to_string(),
                "varchar".to_string(),
                "varchar(255)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(255),
        ];

        let map1 = build_schema_map(columns1);
        let map2 = build_schema_map(columns2);

        let diff = compare_schema_maps(&map1, &map2);
        let sql_statements = generate_sql_diff(&diff);

        // Verify SQL statements include schema names
        assert!(sql_statements.values().any(|s| s.contains("ALTER TABLE `public`.users DROP COLUMN name")));
        assert!(sql_statements.values().any(|s| s.contains("ALTER TABLE `public`.users ADD COLUMN email")));
    }
}
