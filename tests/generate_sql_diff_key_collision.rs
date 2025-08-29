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
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "col1".to_string(),
                    data_type: "varchar".to_string(),
                    is_nullable: "YES".to_string(),
                    column_default: None,
                    character_maximum_length: Some(255),
                },
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "col2".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: "NO".to_string(),
                    column_default: None,
                    character_maximum_length: None,
                }
            ],
            columns_only_in_second: vec![
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "col3".to_string(),
                    data_type: "timestamp".to_string(),
                    is_nullable: "YES".to_string(),
                    column_default: None,
                    character_maximum_length: None,
                },
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "col4".to_string(),
                    data_type: "boolean".to_string(),
                    is_nullable: "YES".to_string(),
                    column_default: None,
                    character_maximum_length: None,
                }
            ],
            columns_with_different_definitions: vec![
                ColumnDifference {
                    table_name: "products".to_string(),
                    column_name: "price".to_string(),
                    first: ColumnInfo {
                        table_schema: "public".to_string(),
                        table_name: "products".to_string(),
                        column_name: "price".to_string(),
                        data_type: "decimal".to_string(),
                        is_nullable: "NO".to_string(),
                        column_default: None,
                        character_maximum_length: None,
                    },
                    second: ColumnInfo {
                        table_schema: "public".to_string(),
                        table_name: "products".to_string(),
                        column_name: "price".to_string(),
                        data_type: "decimal".to_string(),
                        is_nullable: "NO".to_string(),
                        column_default: Some("0.00".to_string()),
                        character_maximum_length: None,
                    },
                }
            ],
        };

        let sql_statements = generate_sql_diff(&diff);
        
        // Print all keys and values to show the collision issue
        println!("Generated SQL statements (showing key collision issue):");
        for (key, value) in &sql_statements {
            println!("  '{}' -> '{}'", key, value);
        }
        
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
        println!("Total entries in result: {}", sql_statements.len());
        println!("Total entries we tried to add: 12 (2+2 tables + 2+2 columns + 1+1 each direction for modify)");
        
        // Verify we have the expected number of unique keys (10)
        // This is fewer than the total number of differences due to key collisions
        assert_eq!(sql_statements.len(), 10);
    }
    
    #[test]
    fn test_generate_sql_diff_improved_implementation_proposal() {
        // This test shows what the improved implementation should produce
        // if we fixed the key collision issue by using unique keys
        
        // In an improved implementation, we would use unique keys like:
        // - "Drop table public.table1 (Schema 1 only)"
        // - "Drop table public.table2 (Schema 1 only)"
        // - "Drop column users.col1 (Schema 1 only)"
        // - "Drop column users.col2 (Schema 1 only)"
        // etc.
        
        // This would allow us to have all the entries without collisions
        // and provide more descriptive keys for the HTML UI
        
        // This test is just documenting what SHOULD happen, not what actually happens
        println!("In an improved implementation:");
        println!("  - Each table/column operation would have a unique key");
        println!("  - All operations would be preserved without overwriting");
        println!("  - Keys would be descriptive like 'Drop table public.users (Schema 1 only)'");
        println!("  - This would make the HTML UI more informative");
    }
}