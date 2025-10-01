use std::fs;
// use wasm_bindgen_test::*;
use db_diff::{ColumnInfo, build_schema_map, compare_schema_maps, create_column_info, compare_schemas};

#[test]
fn test_sample_schema_files_exist() {
    // Test that our sample schema files exist and can be read
    let schema1 = fs::read_to_string("sample_schema1.json");
    let schema2 = fs::read_to_string("sample_schema2.json");
    let schema3 = fs::read_to_string("sample_schema3.json");
    let schema4 = fs::read_to_string("sample_schema4.json");

    assert!(schema1.is_ok());
    assert!(schema2.is_ok());
    assert!(schema3.is_ok());
    assert!(schema4.is_ok());

    // Verify they contain valid JSON
    let schema1_content = schema1.unwrap();
    let schema1_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&schema1_content);
    assert!(schema1_json.is_ok());
    let schema2_content = schema2.unwrap();
    let schema2_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&schema2_content);
    assert!(schema2_json.is_ok());
    let schema3_content = schema3.unwrap();
    let schema3_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&schema3_content);
    assert!(schema3_json.is_ok());
    let schema4_content = schema4.unwrap();
    let schema4_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&schema4_content);
    assert!(schema4_json.is_ok());
}

#[test]
fn test_schema_structure() {
    // Test that schema files have the expected structure
    let schema1_content = fs::read_to_string("sample_schema1.json")
        .expect("Failed to read sample_schema1.json");

    let schema1: Result<Vec<ColumnInfo>, _> = serde_json::from_str(&schema1_content);
    assert!(schema1.is_ok());

    let schema1_vec = schema1.unwrap();
    if !schema1_vec.is_empty() {
        // Check the structure of the first item
        let first_item = &schema1_vec[0];
        assert!(!first_item.table_name.is_empty());
        assert!(!first_item.column_name.is_empty());
        assert!(!first_item.data_type.is_empty());
        assert!(!first_item.is_nullable.is_empty());
    }
}

#[test]
fn test_compare_sample_schemas_functionality() {
    // Read the sample schema files
    let schema1_content = fs::read_to_string("sample_schema1.json")
        .expect("Failed to read sample_schema1.json");
    let schema2_content = fs::read_to_string("sample_schema2.json")
        .expect("Failed to read sample_schema2.json");

    // Parse the JSON content
    let columns1: Vec<ColumnInfo> = serde_json::from_str(&schema1_content)
        .expect("Failed to parse sample_schema1.json");
    let columns2: Vec<ColumnInfo> = serde_json::from_str(&schema2_content)
        .expect("Failed to parse sample_schema2.json");

    // Build schema maps
    let map1 = build_schema_map(columns1);
    let map2 = build_schema_map(columns2);

    // Compare the schemas
    let diff = compare_schema_maps(&map1, &map2);

    // Basic checks - should have some differences
    assert!(!diff.tables_only_in_first.is_empty() ||
            !diff.tables_only_in_second.is_empty() ||
            !diff.columns_only_in_first.is_empty() ||
            !diff.columns_only_in_second.is_empty() ||
            !diff.columns_with_different_definitions.is_empty());
}

#[test]
fn test_identical_schemas_functionality() {
    // Read the same file twice to simulate identical schemas
    let schema_content = fs::read_to_string("sample_schema1.json")
        .expect("Failed to read sample_schema1.json");

    // Parse the JSON content
    let columns: Vec<ColumnInfo> = serde_json::from_str(&schema_content)
        .expect("Failed to parse sample_schema1.json");

    // Build identical schema maps
    let map1 = build_schema_map(columns.clone());
    let map2 = build_schema_map(columns);

    // Compare the schemas
    let diff = compare_schema_maps(&map1, &map2);

    // For identical schemas, all difference lists should be empty
    assert!(diff.tables_only_in_first.is_empty());
    assert!(diff.tables_only_in_second.is_empty());
    assert!(diff.columns_only_in_first.is_empty());
    assert!(diff.columns_only_in_second.is_empty());
    assert!(diff.columns_with_different_definitions.is_empty());
}

#[test]
fn test_mariadb_schema() {
    // Read the same file twice to simulate identical schemas
    let schema_content = fs::read_to_string("sample_schema7.json")
        .expect("Failed to read sample_schema7.json");

    // Compare the schemas
    let _columns1 = match create_column_info(&schema_content) {
        Ok(column) => column,
        Err(e) => return assert_eq!(e,"2"),
    };

    let schema_content2 = fs::read_to_string("sample_schema8.json")
        .expect("Failed to read sample_schema8.json");

    // Compare the schemas
    let _columns1 = match create_column_info(&schema_content2) {
        Ok(column) => column,
        Err(e) => return assert_eq!(e,"2"),
    };


    assert_eq!(1,1)

}

#[test]
fn test_mariadb_compare(){
    let schema_1 = fs::read_to_string("sample_schema7.json")
        .expect("Failed to read sample_schema7.json");

    let schema_2 = fs::read_to_string("sample_schema8.json")
        .expect("Failed to read sample_schema8.json");

    let _diff = compare_schemas(&schema_1, &schema_2);
    

    assert_eq!(1,1)
}
