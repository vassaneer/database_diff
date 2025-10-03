#[cfg(test)]
mod index_diff_tests {
    use std::fs;
    use db_diff::index::{IndexInfo, IndexDifference, IndexDiff, compare_index_maps, build_index_map, create_index_info, generate_sql_index_diff, compare_indexs};

    #[test]
    fn test_index_info_creation() {
        let index_info = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("YES")
            .index_type("BTREE")
            .seq_in_index(1)
            .collation_opt("A".to_string())
            .cardinality_opt(100)
            .build()
            .expect("Build should succeed with all required fields");

        assert_eq!(index_info.table_schema, "public");
        assert_eq!(index_info.table_name, "users");
        assert_eq!(index_info.index_name, "idx_users_email");
        assert_eq!(index_info.column_name, "email");
        assert_eq!(index_info.nullable, "YES");
        assert_eq!(index_info.index_type, "BTREE");
        assert_eq!(index_info.seq_in_index, Some(1));
        assert_eq!(index_info.collation, Some("A".to_string()));
        assert_eq!(index_info.cardinality, Some(100));
    }

    #[test]
    fn test_build_index_map() {
        let indexes = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_name")
                .column_name("name")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let index_map = build_index_map(indexes);

        // Check that we have one table key
        assert_eq!(index_map.len(), 1);

        let table_key = "`public`.users";
        assert!(index_map.contains_key(table_key));

        let table_indexes = index_map.get(table_key).unwrap();
        assert_eq!(table_indexes.len(), 2);
        assert!(table_indexes.contains_key("idx_users_email"));
        assert!(table_indexes.contains_key("idx_users_name"));
    }

    #[test]
    fn test_compare_identical_indexes() {
        let indexes1 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let indexes2 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);

        assert!(diff.indexes_only_in_first.is_empty());
        assert!(diff.indexes_only_in_second.is_empty());
        assert!(diff.indexes_with_different_definitions.is_empty());
    }

    #[test]
    fn test_compare_indexes_only_in_first() {
        let indexes1 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let indexes2 = vec![]; // No indexes in schema 2

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);

        assert_eq!(diff.indexes_only_in_first.len(), 1);
        assert!(diff.indexes_only_in_second.is_empty());
        assert!(diff.indexes_with_different_definitions.is_empty());

        assert_eq!(diff.indexes_only_in_first[0].index_name, "idx_users_email");
    }

    #[test]
    fn test_compare_indexes_only_in_second() {
        let indexes1 = vec![]; // No indexes in schema 1

        let indexes2 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);

        assert!(diff.indexes_only_in_first.is_empty());
        assert_eq!(diff.indexes_only_in_second.len(), 1);
        assert!(diff.indexes_with_different_definitions.is_empty());

        assert_eq!(diff.indexes_only_in_second[0].index_name, "idx_users_email");
    }

    #[test]
    fn test_compare_indexes_with_different_definitions() {
        let indexes1 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let indexes2 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("NO") // Different nullable value
                .index_type("BTREE")
                .build()
                .unwrap(),
        ];

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);

        assert!(diff.indexes_only_in_first.is_empty());
        assert!(diff.indexes_only_in_second.is_empty());
        assert_eq!(diff.indexes_with_different_definitions.len(), 1);

        let diff_item = &diff.indexes_with_different_definitions[0];
        assert_eq!(diff_item.index_name, "idx_users_email");
        assert_eq!(diff_item.first.nullable, "YES");
        assert_eq!(diff_item.second.nullable, "NO");
    }

    #[test]
    fn test_index_difference_creation() {
        let index1 = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("YES")
            .index_type("BTREE")
            .build()
            .unwrap();

        let index2 = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("NO")
            .index_type("BTREE")
            .build()
            .unwrap();

        let difference = IndexDifference {
            table_name: "users".to_string(),
            index_name: "idx_users_email".to_string(),
            first: index1,
            second: index2,
        };

        assert_eq!(difference.table_name, "users");
        assert_eq!(difference.index_name, "idx_users_email");
        assert_eq!(difference.first.nullable, "YES");
        assert_eq!(difference.second.nullable, "NO");
    }

    #[test]
    fn test_create_index_info_from_json() {
        let json_data = r#"[
            {
                "table_schema": "public",
                "table_name": "users",
                "index_name": "idx_users_email",
                "column_name": "email",
                "nullable": "YES",
                "index_type": "BTREE"
            }
        ]"#;

        let indexes = create_index_info(json_data).expect("Should parse JSON successfully");
        assert_eq!(indexes.len(), 1);
        assert_eq!(indexes[0].table_schema, "public");
        assert_eq!(indexes[0].table_name, "users");
        assert_eq!(indexes[0].index_name, "idx_users_email");
    }

    #[test]
    fn test_generate_sql_index_diff() {
        let diff = IndexDiff {
            indexes_only_in_first: vec![
                IndexInfo::builder()
                    .table_schema("public")
                    .table_name("users")
                    .index_name("idx_users_email")
                    .column_name("email")
                    .nullable("YES")
                    .index_type("BTREE")
                    .build()
                    .unwrap(),
            ],
            indexes_only_in_second: vec![
                IndexInfo::builder()
                    .table_schema("public")
                    .table_name("users")
                    .index_name("idx_users_name")
                    .column_name("name")
                    .nullable("YES")
                    .index_type("BTREE")
                    .build()
                    .unwrap(),
            ],
            indexes_with_different_definitions: vec![],
        };

        let sql_statements = generate_sql_index_diff(&diff);

        // Check that we have SQL statements for dropping indexes from schema 1
        let drop_statements = sql_statements.get("Drop index in Schema 1 (Schema 1 only)");
        assert!(drop_statements.is_some());
        assert!(drop_statements.unwrap().contains("DROP INDEX `idx_users_email` ON `public`.users;"));

        // Check that we have SQL statements for adding indexes in schema 2
        let add_statements = sql_statements.get("Add index in Schema 1 (Schema 2 only)");
        assert!(add_statements.is_some());
        assert!(add_statements.unwrap().contains("CREATE INDEX `idx_users_name` ON `public`.users (name);"));
    }

    #[test]
    fn test_index_comparison_with_non_unique_field() {
        // Testing that indexes with different non_unique values are detected as different
        let indexes1 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .non_unique_opt(0)  // Unique index (0)
                .build()
                .unwrap(),
        ];

        let indexes2 = vec![
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_email")
                .column_name("email")
                .nullable("YES")
                .index_type("BTREE")
                .non_unique_opt(1)  // Non-unique index (1)
                .build()
                .unwrap(),
        ];

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);
        let sql_statements = generate_sql_index_diff(&diff);
        // println!("{:?}",sql_statements);

        // The indexes should be detected as different due to non_unique field
        assert!(diff.indexes_only_in_first.is_empty());
        assert!(diff.indexes_only_in_second.is_empty());
        assert_eq!(diff.indexes_with_different_definitions.len(), 1);

        let diff_item = &diff.indexes_with_different_definitions[0];
        assert_eq!(diff_item.index_name, "idx_users_email");
        assert_eq!(diff_item.first.non_unique, Some(0));  // Unique in first schema
        assert_eq!(diff_item.second.non_unique, Some(1)); // Non-unique in second schema
        let add_statements = sql_statements.get("Modify index in Schema 2");
        assert!(add_statements.unwrap().contains("CREATE UNIQUE INDEX `idx_users_email` ON `public`.users (email);"));
    }

    #[test]
    fn test_multi_key_unique_with_seq_in_index() {
        // Testing multi-key unique indexes with proper seq_in_index handling
        // This test verifies that multi-key unique indexes are correctly compared when the order of columns differs
        let indexes1 = vec![
            // First column of unique multi-key index (email first)
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_unique_multi")
                .column_name("email")
                .seq_in_index(1)  // First column in the index
                .nullable("NO")
                .index_type("BTREE")
                .non_unique_opt(0)  // Unique index (0)
                .build()
                .unwrap(),
            // Second column of unique multi-key index (username second)
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_unique_multi")
                .column_name("username")
                .seq_in_index(2)  // Second column in the index
                .nullable("NO")
                .index_type("BTREE")
                .non_unique_opt(0)  // Unique index (0)
                .build()
                .unwrap(),
        ];

        let indexes2 = vec![
            // Same multi-key unique index but with reversed column order (username first)
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_unique_multi")
                .column_name("username")
                .seq_in_index(1)  // Now first column in the index
                .nullable("NO")
                .index_type("BTREE")
                .non_unique_opt(0)  // Unique index (0)
                .build()
                .unwrap(),
            // Second column in new order (email second)
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_unique_multi")
                .column_name("email")
                .seq_in_index(2)  // Now second column in the index
                .nullable("NO")
                .index_type("BTREE")
                .non_unique_opt(0)  // Unique index (0)
                .build()
                .unwrap(),
        ];

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);
        let sql_statements = generate_sql_index_diff(&diff);

        // Debug print to see what SQL statements are generated
        // println!("SQL statements: {:?}", sql_statements);
        // assert_eq!(1,2);

        // The indexes should be detected as different due to seq_in_index order (column sequence)
        assert!(diff.indexes_only_in_first.is_empty());
        assert!(diff.indexes_only_in_second.is_empty());
        assert_eq!(diff.indexes_with_different_definitions.len(), 1);

        let diff_item = &diff.indexes_with_different_definitions[0];
        assert_eq!(diff_item.index_name, "idx_users_unique_multi");

        // Verify that the SQL generation properly handles unique multi-column indexes
        let modify_statements = sql_statements.get("Modify index in Schema 2");
        assert!(modify_statements.is_some());
        let statement = modify_statements.unwrap();
        // The statement should contain operations to modify the index due to different column sequence
        assert!(statement.contains("CREATE UNIQUE INDEX") || statement.contains("DROP INDEX") || statement.contains("CREATE INDEX"));
        assert!(statement.contains("idx_users_unique_multi"));

        // Both schemas have the same columns in the multi-key unique index, but in different order
        // This test confirms that seq_in_index is properly considered during index comparison
    }

    #[test]
    fn test_multi_column_index_comparison() {
        // Testing complex scenario with multi-column indexes that have different column orders
        let indexes1 = vec![
            // First index with two columns
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_multi")
                .column_name("email")
                .seq_in_index(1) // First column in index
                .nullable("YES")
                .index_type("BTREE")
                .non_unique_opt(1)
                .build()
                .unwrap(),
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_multi")
                .column_name("created_at")
                .seq_in_index(2) // Second column in index
                .nullable("NO")
                .index_type("BTREE")
                .non_unique_opt(1)
                .build()
                .unwrap(),
        ];

        let indexes2 = vec![
            // Same index but with reversed column order
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_multi")
                .column_name("created_at")
                .seq_in_index(1) // First column in index (different order)
                .nullable("NO")
                .index_type("BTREE")
                .non_unique_opt(1)
                .build()
                .unwrap(),
            IndexInfo::builder()
                .table_schema("public")
                .table_name("users")
                .index_name("idx_users_multi")
                .column_name("email")
                .seq_in_index(2) // Second column in index (different order)
                .nullable("YES")
                .index_type("BTREE")
                .non_unique_opt(1)
                .build()
                .unwrap(),
        ];

        let index_map1 = build_index_map(indexes1);
        let index_map2 = build_index_map(indexes2);

        let diff = compare_index_maps(&index_map1, &index_map2);
        // let sql_statements = generate_sql_index_diff(&diff);
        // println!("{:?}",sql_statements);
        // Even though both schemas have the same columns in the index,
        // the order is different, so this should be detected as a difference
        assert!(diff.indexes_only_in_first.is_empty());
        assert!(diff.indexes_only_in_second.is_empty());
        assert_eq!(diff.indexes_with_different_definitions.len(), 1);

        let diff_item = &diff.indexes_with_different_definitions[0];
        assert_eq!(diff_item.index_name, "idx_users_multi");
        // The comparison logic should detect that the multi-column index has different ordering
    }

    #[test]
    fn test_index_file() {
        let schema1 = fs::read_to_string("sample_index1.json");
        assert!(schema1.is_ok());
        let schema1_content = schema1.unwrap();
        // Parse the outer JSON structure to extract the "output" string
        let schema1_json = create_index_info(&schema1_content);
        assert!(schema1_json.is_ok());
    }

    #[test]
    fn test_index_multi_file() {
        let schema1 = fs::read_to_string("sample_index1.json");
        assert!(schema1.is_ok());
        let schema1_content = schema1.unwrap();
        let schema2 = fs::read_to_string("sample_index2.json");
        assert!(schema2.is_ok());
        let schema2_content = schema2.unwrap();
        let _diff = compare_indexs(&schema1_content, &schema2_content);
        println!("{:?}", _diff);
        // assert!(schema2.is_ok());
        // Parse the outer JSON structure to extract the "output" string
        // let schema1_json = create_index_info(&schema1_content);
        // assert!(schema1_json.is_ok());
    }
}
