#[cfg(test)]
mod tests {
    use db_diff::{ColumnInfo, build_schema_map, compare_schema_maps, generate_sql_diff};

    #[test]
    fn test_sql_generation_includes_schema_names() {
        // Create test data with schema information
        let columns1 = vec![
            ColumnInfo {
                table_schema: "public".to_string(),
                table_name: "users".to_string(),
                column_name: "id".to_string(),
                data_type: "integer".to_string(),
                is_nullable: "NO".to_string(),
                column_default: None,
                character_maximum_length: None,
            },
            ColumnInfo {
                table_schema: "public".to_string(),
                table_name: "users".to_string(),
                column_name: "name".to_string(),
                data_type: "varchar".to_string(),
                is_nullable: "YES".to_string(),
                column_default: None,
                character_maximum_length: Some(255),
            },
        ];

        let columns2 = vec![
            ColumnInfo {
                table_schema: "public".to_string(),
                table_name: "users".to_string(),
                column_name: "id".to_string(),
                data_type: "integer".to_string(),
                is_nullable: "NO".to_string(),
                column_default: None,
                character_maximum_length: None,
            },
            ColumnInfo {
                table_schema: "public".to_string(),
                table_name: "users".to_string(),
                column_name: "email".to_string(),
                data_type: "varchar".to_string(),
                is_nullable: "YES".to_string(),
                column_default: None,
                character_maximum_length: Some(255),
            },
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

        // Verify that ALTER TABLE statements include schema names
        let has_drop_with_schema = sql_statements.values().any(|s| s.contains("ALTER TABLE public.users DROP COLUMN name"));
        let has_add_with_schema = sql_statements.values().any(|s| s.contains("ALTER TABLE public.users ADD COLUMN email"));
        
        assert!(has_drop_with_schema, "Should have ALTER TABLE statement with schema name for DROP COLUMN");
        assert!(has_add_with_schema, "Should have ALTER TABLE statement with schema name for ADD COLUMN");
    }
}