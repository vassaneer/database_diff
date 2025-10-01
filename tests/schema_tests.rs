// use wasm_bindgen_test::*;
use db_diff::{ColumnInfo, build_schema_map, compare_schema_maps};
use serde_json;

#[test]
fn test_build_schema_map() {
    let columns = vec![
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
        ColumnInfo::builder(
            "public".to_string(),
            "products".to_string(),
            "id".to_string(),
            "integer".to_string(),
            "int(11)".to_string(),
            "NO".to_string(),
        ),
    ];

    let schema_map = build_schema_map(columns);

    // Should have 2 tables
    assert_eq!(schema_map.len(), 2);

    // Users table should have 2 columns
    assert_eq!(schema_map.get("`public`.users").unwrap().len(), 2);

    // Products table should have 1 column
    assert_eq!(schema_map.get("`public`.products").unwrap().len(), 1);

    // Check specific column details
    let users_id = &schema_map.get("`public`.users").unwrap().get("id").unwrap();
    assert_eq!(users_id.data_type, "integer");

    let users_name = &schema_map.get("`public`.users").unwrap().get("name").unwrap();
    assert_eq!(users_name.character_maximum_length, Some(255));
}

#[test]
fn test_compare_schema_maps_identical() {
    let columns = vec![
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

    let map1 = build_schema_map(columns.clone());
    let map2 = build_schema_map(columns);

    let diff = compare_schema_maps(&map1, &map2);

    // Should have no differences
    assert_eq!(diff.tables_only_in_first.len(), 0);
    assert_eq!(diff.tables_only_in_second.len(), 0);
    assert_eq!(diff.columns_only_in_first.len(), 0);
    assert_eq!(diff.columns_only_in_second.len(), 0);
    assert_eq!(diff.columns_with_different_definitions.len(), 0);
}

#[test]
fn test_compare_schema_maps_with_differences() {
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

    // Should have no tables only in first or second
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
fn test_compare_schema_maps_different_tables() {
    let columns1 = vec![
        ColumnInfo::builder(
            "public".to_string(),
            "users".to_string(),
            "id".to_string(),
            "integer".to_string(),
            "int(11)".to_string(),
            "NO".to_string(),
        )
    ];

    let columns2 = vec![
        ColumnInfo::builder(
            "public".to_string(),
            "products".to_string(),
            "id".to_string(),
            "integer".to_string(),
            "int(11)".to_string(),
            "NO".to_string(),
        ),
    ];

    let map1 = build_schema_map(columns1);
    let map2 = build_schema_map(columns2);

    let diff = compare_schema_maps(&map1, &map2);

    // Should have tables only in first and second
    assert_eq!(diff.tables_only_in_first.len(), 1);
    assert_eq!(diff.tables_only_in_second.len(), 1);
    
    assert_eq!(diff.tables_only_in_first[0].0, "`public`.users");
    assert_eq!(diff.tables_only_in_second[0].0, "`public`.products");
    assert_eq!(diff.tables_only_in_first[0].1, "id int(11) NOT NULL,");
    assert_eq!(diff.tables_only_in_second[0].1, "id int(11) NOT NULL,");

    // Should have no column differences
    assert_eq!(diff.columns_only_in_first.len(), 0);
    assert_eq!(diff.columns_only_in_second.len(), 0);
    assert_eq!(diff.columns_with_different_definitions.len(), 0);
}

#[test]
fn test_character_maximum_length_parsing() {
    // Test JSON with integer representation
    let json_int = r#"[{
        "table_schema": "public",
        "table_name": "users",
        "column_name": "name",
        "data_type": "varchar",
        "column_type":"varchar(255)",
        "is_nullable": "YES",
        "column_default": null,
        "character_maximum_length": 255
    }]"#;

    // Test JSON with string representation
    let json_str = r#"[{
        "table_schema": "public",
        "table_name": "users",
        "column_name": "name",
        "data_type": "varchar",
        "column_type":"varchar(255)",
        "is_nullable": "YES",
        "column_default": null,
        "character_maximum_length": "255"
    }]"#;

    // Test JSON with large integer representation
    let json_large = r#"[{
        "table_schema": "public",
        "table_name": "posts",
        "column_name": "content",
        "data_type": "longtext",
        "column_type":"longtext",
        "is_nullable": "YES",
        "column_default": null,
        "character_maximum_length": 4294967295
    }]"#;

    // Test JSON with large string representation
    let json_large_str = r#"[{
        "table_schema": "public",
        "table_name": "posts",
        "column_name": "content",
        "data_type": "longtext",
        "column_type":"longtext",
        "is_nullable": "YES",
        "column_default": null,
        "character_maximum_length": "4294967295"
    }]"#;

    // Test JSON with null representation
    let json_null = r#"[{
        "table_schema": "public",
        "table_name": "users",
        "column_name": "id",
        "data_type": "integer",
        "column_type":"int(11)",
        "is_nullable": "NO",
        "column_default": null,
        "character_maximum_length": null
    }]"#;

    // All should parse successfully
    let result_int: Result<Vec<ColumnInfo>, _> = serde_json::from_str(json_int);
    let result_str: Result<Vec<ColumnInfo>, _> = serde_json::from_str(json_str);
    let result_large: Result<Vec<ColumnInfo>, _> = serde_json::from_str(json_large);
    let result_large_str: Result<Vec<ColumnInfo>, _> = serde_json::from_str(json_large_str);
    let result_null: Result<Vec<ColumnInfo>, _> = serde_json::from_str(json_null);

    // Verify all parsing was successful
    assert!(result_int.is_ok());
    assert!(result_str.is_ok());
    assert!(result_large.is_ok());
    assert!(result_large_str.is_ok());
    assert!(result_null.is_ok());

    // Verify the parsed values
    let columns_int = result_int.unwrap();
    let columns_str = result_str.unwrap();
    let columns_large = result_large.unwrap();
    let columns_large_str = result_large_str.unwrap();
    let columns_null = result_null.unwrap();

    // Both should have the same value for character_maximum_length
    assert_eq!(columns_int[0].character_maximum_length, Some(255));
    assert_eq!(columns_str[0].character_maximum_length, Some(255));
    assert_eq!(columns_large[0].character_maximum_length, Some(4294967295));
    assert_eq!(columns_large_str[0].character_maximum_length, Some(4294967295));
    assert_eq!(columns_null[0].character_maximum_length, None);
}

#[test]
fn test_generate_sql_diff() {
    use db_diff::{SchemaDiff, ColumnDifference, generate_sql_diff};

    // Create a test diff with multiple tables
    let diff = SchemaDiff {
        tables_only_in_first: vec![("`public`.old_table".to_string(), "old_table".to_string())],
        tables_only_in_second: vec![("`public`.new_table".to_string(), "new_table".to_string())],
        columns_only_in_first: vec![
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "old_column".to_string(),
                "varchar".to_string(),
                "varchar(255)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(255),
            ColumnInfo::builder(
                "public".to_string(),
                "products".to_string(),
                "discontinued".to_string(),
                "boolean".to_string(),
                "boolean".to_string(),
                "YES".to_string(),
            )
            .set_default("false".to_string())
        ],
        columns_only_in_second: vec![
            ColumnInfo::builder(
                "public".to_string(),
                "users".to_string(),
                "new_column".to_string(),
                "varchar".to_string(),
                "varchar(100)".to_string(),
                "NO".to_string(),
            )
            .set_character_maximum_length(100)
            .set_default("default_value".to_string()),
            ColumnInfo::builder(
                "public".to_string(),
                "orders".to_string(),
                "tracking_number".to_string(),
                "varchar".to_string(),
                "varchar(50)".to_string(),
                "YES".to_string(),
            )
            .set_character_maximum_length(50)
        ],
        columns_with_different_definitions: vec![
            ColumnDifference {
                table_name: "users".to_string(),
                column_name: "changed_column".to_string(),
                first: ColumnInfo::builder(
                    "public".to_string(),
                    "users".to_string(),
                    "changed_column".to_string(),
                    "int".to_string(),
                    "int".to_string(),
                    "NO".to_string(),
                ),
                second: ColumnInfo::builder(
                    "public".to_string(),
                    "users".to_string(),
                    "changed_column".to_string(),
                    "bigint".to_string(),
                    "bigint".to_string(),
                    "NO".to_string(),
                ),
            },
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
                ),
            }
        ],
    };

    // Generate SQL statements
    let sql_statements = generate_sql_diff(&diff);

    // Convert HashMap values to a Vec for easier searching
    let statements: Vec<&String> = sql_statements.values().collect();

    // Verify we have statements
    assert!(!sql_statements.is_empty());

    // Verify specific statements exist for schema 1 -> schema 2 transformation
    assert!(statements.iter().any(|s| s.contains("DROP TABLE `public`.old_table")));
    assert!(statements.iter().any(|s| s.contains("-- CREATE TABLE `public`.new_table (new_table)")));

    // Verify specific statements exist for schema 2 -> schema 1 transformation
    assert!(statements.iter().any(|s| s.contains("DROP TABLE `public`.new_table")));
    assert!(statements.iter().any(|s| s.contains("-- CREATE TABLE `public`.old_table (old_table)")));

    // Verify specific column operations with schema names
    assert!(statements.iter().any(|s| s.contains("ALTER TABLE `public`.users DROP COLUMN old_column")));
    assert!(statements.iter().any(|s| s.contains("ALTER TABLE `public`.users ADD COLUMN new_column")));
    assert!(statements.iter().any(|s| s.contains("ALTER TABLE `public`.users MODIFY COLUMN changed_column")));
    assert!(statements.iter().any(|s| s.contains("ALTER TABLE `public`.products DROP COLUMN discontinued")));
    assert!(statements.iter().any(|s| s.contains("ALTER TABLE `public`.orders ADD COLUMN tracking_number")));
}
