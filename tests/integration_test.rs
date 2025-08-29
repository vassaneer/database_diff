use std::fs;

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
}

#[test]
fn test_schema_structure() {
    // Test that schema files have the expected structure
    let schema1_content = fs::read_to_string("sample_schema1.json")
        .expect("Failed to read sample_schema1.json");

    // Verify they contain valid JSON
    let schema1_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&schema1_content);
    assert!(schema1_json.is_ok());

    let schema1_vec = schema1_json.unwrap();
    if !schema1_vec.is_empty() {
        // Check the structure of the first item
        let first_item = &schema1_vec[0];
        assert!(first_item.get("table_name").is_some());
        assert!(first_item.get("column_name").is_some());
        assert!(first_item.get("data_type").is_some());
        assert!(first_item.get("is_nullable").is_some());
    }
}
