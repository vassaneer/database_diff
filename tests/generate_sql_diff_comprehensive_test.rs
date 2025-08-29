#[cfg(test)]
mod tests {
    use db_diff::{SchemaDiff, ColumnInfo, ColumnDifference, generate_sql_diff};

    #[test]
    fn test_generate_sql_diff_with_empty_diff() {
        let diff = SchemaDiff {
            tables_only_in_first: vec![],
            tables_only_in_second: vec![],
            columns_only_in_first: vec![],
            columns_only_in_second: vec![],
            columns_with_different_definitions: vec![],
        };

        let sql_statements = generate_sql_diff(&diff);
        // Should return an empty HashMap when there are no differences
        assert!(sql_statements.is_empty());
    }

    #[test]
    fn test_generate_sql_diff_with_table_differences() {
        let diff = SchemaDiff {
            tables_only_in_first: vec![
                ("public".to_string(), "old_table".to_string()),
            ],
            tables_only_in_second: vec![
                ("public".to_string(), "new_table".to_string()),
            ],
            columns_only_in_first: vec![],
            columns_only_in_second: vec![],
            columns_with_different_definitions: vec![],
        };

        let sql_statements = generate_sql_diff(&diff);
        
        // Should have statements for dropping and creating tables
        assert!(sql_statements.contains_key("Drop table in Schema 1 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Create table in Schema 1 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Drop table in Schema 2 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Create table in Schema 2 (Schema 1 only)"));
        
        // Check the actual SQL statements
        assert_eq!(
            sql_statements.get("Drop table in Schema 1 (Schema 1 only)").unwrap(),
            "DROP TABLE public.old_table;"
        );
        assert_eq!(
            sql_statements.get("Create table in Schema 1 (Schema 2 only)").unwrap(),
            "-- CREATE TABLE public.new_table (...);"
        );
    }

    #[test]
    fn test_generate_sql_diff_with_column_differences() {
        let diff = SchemaDiff {
            tables_only_in_first: vec![],
            tables_only_in_second: vec![],
            columns_only_in_first: vec![
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "old_column".to_string(),
                    data_type: "varchar".to_string(),
                    is_nullable: "YES".to_string(),
                    column_default: None,
                    character_maximum_length: Some(255),
                }
            ],
            columns_only_in_second: vec![
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "new_column".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: "NO".to_string(),
                    column_default: Some("0".to_string()),
                    character_maximum_length: None,
                }
            ],
            columns_with_different_definitions: vec![],
        };

        let sql_statements = generate_sql_diff(&diff);
        
        // Should have statements for dropping and adding columns
        assert!(sql_statements.contains_key("Drop column in Schema 1 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Add column in Schema 1 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Add column in Schema 2 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Drop column in Schema 2 (Schema 2 only)"));
        
        // Check the actual SQL statements
        assert_eq!(
            sql_statements.get("Drop column in Schema 1 (Schema 1 only)").unwrap(),
            "ALTER TABLE public.users DROP COLUMN old_column;"
        );
        assert!(sql_statements.get("Add column in Schema 1 (Schema 2 only)").unwrap()
            .contains("ALTER TABLE public.users ADD COLUMN new_column integer NOT NULL DEFAULT 0"));
    }

    #[test]
    fn test_generate_sql_diff_with_column_modifications() {
        let diff = SchemaDiff {
            tables_only_in_first: vec![],
            tables_only_in_second: vec![],
            columns_only_in_first: vec![],
            columns_only_in_second: vec![],
            columns_with_different_definitions: vec![
                ColumnDifference {
                    table_name: "users".to_string(),
                    column_name: "modified_column".to_string(),
                    first: ColumnInfo {
                        table_schema: "public".to_string(),
                        table_name: "users".to_string(),
                        column_name: "modified_column".to_string(),
                        data_type: "varchar".to_string(),
                        is_nullable: "YES".to_string(),
                        column_default: None,
                        character_maximum_length: Some(255),
                    },
                    second: ColumnInfo {
                        table_schema: "public".to_string(),
                        table_name: "users".to_string(),
                        column_name: "modified_column".to_string(),
                        data_type: "varchar".to_string(),
                        is_nullable: "NO".to_string(),
                        column_default: Some("''".to_string()),
                        character_maximum_length: Some(100),
                    },
                }
            ],
        };

        let sql_statements = generate_sql_diff(&diff);
        
        // Should have statements for modifying columns
        assert!(sql_statements.contains_key("Modify column in Schema 1"));
        assert!(sql_statements.contains_key("Modify column in Schema 2"));
        
        // Check the actual SQL statements
        assert!(sql_statements.get("Modify column in Schema 1").unwrap()
            .contains("ALTER TABLE public.users MODIFY COLUMN modified_column varchar(100) NOT NULL DEFAULT ''"));
        assert!(sql_statements.get("Modify column in Schema 2").unwrap()
            .contains("ALTER TABLE public.users MODIFY COLUMN modified_column varchar(255) NULL"));
    }

    #[test]
    fn test_generate_sql_diff_with_multiple_differences() {
        let diff = SchemaDiff {
            tables_only_in_first: vec![
                ("public".to_string(), "table1".to_string()),
            ],
            tables_only_in_second: vec![
                ("public".to_string(), "table2".to_string()),
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
                }
            ],
            columns_only_in_second: vec![
                ColumnInfo {
                    table_schema: "public".to_string(),
                    table_name: "users".to_string(),
                    column_name: "col2".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: "NO".to_string(),
                    column_default: Some("0".to_string()),
                    character_maximum_length: None,
                }
            ],
            columns_with_different_definitions: vec![
                ColumnDifference {
                    table_name: "orders".to_string(),
                    column_name: "status".to_string(),
                    first: ColumnInfo {
                        table_schema: "public".to_string(),
                        table_name: "orders".to_string(),
                        column_name: "status".to_string(),
                        data_type: "varchar".to_string(),
                        is_nullable: "YES".to_string(),
                        column_default: None,
                        character_maximum_length: Some(50),
                    },
                    second: ColumnInfo {
                        table_schema: "public".to_string(),
                        table_name: "orders".to_string(),
                        column_name: "status".to_string(),
                        data_type: "varchar".to_string(),
                        is_nullable: "NO".to_string(),
                        column_default: Some("'pending'".to_string()),
                        character_maximum_length: Some(100),
                    },
                }
            ],
        };

        let sql_statements = generate_sql_diff(&diff);
        
        // Should have statements for all differences
        // Note: Because we're using generic keys, some entries will overwrite others
        // The implementation has a flaw where it uses non-unique keys
        assert!(sql_statements.contains_key("Drop table in Schema 1 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Create table in Schema 1 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Drop column in Schema 1 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Add column in Schema 1 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Modify column in Schema 1"));
        assert!(sql_statements.contains_key("Drop table in Schema 2 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Create table in Schema 2 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Add column in Schema 2 (Schema 1 only)"));
        assert!(sql_statements.contains_key("Drop column in Schema 2 (Schema 2 only)"));
        assert!(sql_statements.contains_key("Modify column in Schema 2"));
        
        // Verify we have the expected number of unique keys
        // Note: This will be fewer than the total number of differences due to key collisions
        assert_eq!(sql_statements.len(), 10);
    }
}