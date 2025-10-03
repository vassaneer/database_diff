use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use wasm_bindgen::prelude::*;
use super::{MariaDBJson};

// Custom deserializer to handle both string and integer representations for u32
fn deserialize_optional_string_as_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(u32),
    }

    let helper = Option::<StringOrInt>::deserialize(deserializer)?;
    match helper {
        Some(StringOrInt::String(s)) => {
            if s.is_empty() || s == "null" {
                Ok(None)
            } else {
                s.parse::<u32>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

// Custom deserializer to handle both string and integer representations for u64
fn deserialize_optional_string_as_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(u64),
    }

    let helper = Option::<StringOrInt>::deserialize(deserializer)?;
    match helper {
        Some(StringOrInt::String(s)) => {
            if s.is_empty() || s == "null" {
                Ok(None)
            } else {
                s.parse::<u64>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

// Custom deserializer to handle both string and integer representations for u8
fn deserialize_optional_string_as_u8<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(u8),
    }

    let helper = Option::<StringOrInt>::deserialize(deserializer)?;
    match helper {
        Some(StringOrInt::String(s)) => {
            if s.is_empty() || s == "null" {
                Ok(None)
            } else {
                s.parse::<u8>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexInfo {
    pub table_schema: String,
    pub table_name: String,
    pub index_name: String,
    pub column_name: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_u32")]
    pub seq_in_index: Option<u32>,
    #[serde(default)]
    pub collation: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_u64")]
    pub cardinality: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_u64")]
    pub sub_part: Option<u64>,
    #[serde(default)]
    pub packed: Option<String>,
    pub nullable: String,
    pub index_type: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_as_u8")]
    pub non_unique: Option<u8>, // 0 for unique, 1 for non-unique
}

impl IndexInfo {
    pub fn builder() -> Self {
        Self {
            table_schema: String::new(),
            table_name: String::new(),
            index_name: String::new(),
            column_name: String::new(),
            seq_in_index: None,
            collation: None,
            cardinality: None,
            sub_part: None,
            packed: None,
            nullable: String::new(),
            index_type: String::new(),
            non_unique: None,
        }
    }

    pub fn table_schema<S: Into<String>>(mut self, table_schema: S) -> Self {
        self.table_schema = table_schema.into();
        self
    }

    pub fn table_name<S: Into<String>>(mut self, table_name: S) -> Self {
        self.table_name = table_name.into();
        self
    }

    pub fn index_name<S: Into<String>>(mut self, index_name: S) -> Self {
        self.index_name = index_name.into();
        self
    }

    pub fn column_name<S: Into<String>>(mut self, column_name: S) -> Self {
        self.column_name = column_name.into();
        self
    }

    pub fn seq_in_index(mut self, seq_in_index: u32) -> Self {
        self.seq_in_index = Some(seq_in_index);
        self
    }

    pub fn collation_opt(mut self, collation: String) -> Self {
        self.collation = Some(collation);
        self
    }

    pub fn cardinality_opt(mut self, cardinality: u64) -> Self {
        self.cardinality = Some(cardinality);
        self
    }

    pub fn sub_part_opt(mut self, sub_part: u64) -> Self {
        self.sub_part = Some(sub_part);
        self
    }

    pub fn packed_opt(mut self, packed: String) -> Self {
        self.packed = Some(packed);
        self
    }

    pub fn non_unique_opt(mut self, non_unique: u8) -> Self {
        self.non_unique = Some(non_unique);
        self
    }

    pub fn nullable<S: Into<String>>(mut self, nullable: S) -> Self {
        self.nullable = nullable.into();
        self
    }

    pub fn index_type<S: Into<String>>(mut self, index_type: S) -> Self {
        self.index_type = index_type.into();
        self
    }

    pub fn build(self) -> Result<Self, String> {
        // Only validate non-Option fields that are not empty
        if self.table_schema.is_empty() {
            return Err("table_schema is required".to_string());
        }
        if self.table_name.is_empty() {
            return Err("table_name is required".to_string());
        }
        if self.index_name.is_empty() {
            return Err("index_name is required".to_string());
        }
        if self.column_name.is_empty() {
            return Err("column_name is required".to_string());
        }
        if self.nullable.is_empty() {
            return Err("nullable is required".to_string());
        }
        if self.index_type.is_empty() {
            return Err("index_type is required".to_string());
        }

        Ok(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexDifference {
    pub table_name: String,
    pub index_name: String,
    pub first: IndexInfo,
    pub second: IndexInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexDiff{
    pub indexes_only_in_first: Vec<IndexInfo>,
    pub indexes_only_in_second: Vec<IndexInfo>,
    pub indexes_with_different_definitions: Vec<IndexDifference>,
}

// What i should do
#[wasm_bindgen]
pub fn compare_indexs(json1: &str, json2: &str) -> String {
    // Parse index information if available
    let indexes1 = match create_index_info(json1) {
        Ok(indexes) => indexes,
        Err(_) => vec![], // If parsing fails, proceed with empty indexes
    };

    let indexes2 = match create_index_info(json2) {
        Ok(indexes) => indexes,
        Err(_) => vec![], // If parsing fails, proceed with empty indexes
    };

    // Create <table, index>
    let index_map1 = build_index_map(indexes1);
    let index_map2 = build_index_map(indexes2);

    // Perform the comparison
    let diff = compare_index_maps(&index_map1, &index_map2);

    // Generate SQL diff statements
    let sql_statements = generate_sql_index_diff(&diff);

    // Create a result object that includes both the diff and SQL statements
    let result = serde_json::json!({
        "diff": diff,
        "sql": sql_statements
    });

    // Serialize the result to JSON
    match serde_json::to_string(&result) {
        Ok(result) => result,
        Err(e) => format!("Error serializing result: {}", e),
    }

}

pub fn compare_index_maps(
    index_map1: &HashMap<String, IndexMap<String, Vec<IndexInfo>>>,
    index_map2: &HashMap<String, IndexMap<String, Vec<IndexInfo>>>
) -> IndexDiff{

    // Compare indexes
    let index_tables1: HashSet<&String> = index_map1.keys().collect();
    let index_tables2: HashSet<&String> = index_map2.keys().collect();

    let mut indexes_only_in_first: Vec<IndexInfo> = Vec::new();
    let mut indexes_only_in_second: Vec<IndexInfo> = Vec::new();
    let mut indexes_with_different_definitions: Vec<IndexDifference> = Vec::new();

    // Find common tables for index comparison
    let common_index_tables: HashSet<&String> = index_tables1.intersection(&index_tables2).cloned().collect();

    for table_key in common_index_tables {
        let indexes1_opt = index_map1.get(table_key);
        let indexes2_opt = index_map2.get(table_key);

        let indexes1 = match indexes1_opt {
            Some(idx_map) => idx_map,
            None => continue,
        };

        let indexes2 = match indexes2_opt {
            Some(idx_map) => idx_map,
            None => continue,
        };

        let index_names1: HashSet<&String> = indexes1.keys().collect();
        let index_names2: HashSet<&String> = indexes2.keys().collect();

        // Indexes only in first schema
        for index_name in index_names1.difference(&index_names2) {
            if let Some(index_list) = indexes1.get(*index_name) {
                for index in index_list {
                    indexes_only_in_first.push(index.clone());
                }
            }
        }

        // Indexes only in second schema
        for index_name in index_names2.difference(&index_names1) {
            if let Some(index_list) = indexes2.get(*index_name) {
                for index in index_list {
                    indexes_only_in_second.push(index.clone());
                }
            }
        }

        // Compare common indexes
        for index_name in index_names1.intersection(&index_names2) {
            let idx1_list = indexes1.get(*index_name).unwrap();
            let idx2_list = indexes2.get(*index_name).unwrap();

            if !compare_index_lists(idx1_list, idx2_list) {
                // Extract just the table name (without schema) for the IndexDifference
                let table_name = table_key.split('.').nth(1).unwrap_or(table_key).to_string();

                let first_idx = idx1_list.get(0).cloned().unwrap_or_else(|| IndexInfo::builder().build().unwrap());
                let second_idx = idx2_list.get(0).cloned().unwrap_or_else(|| IndexInfo::builder().build().unwrap());

                indexes_with_different_definitions.push(IndexDifference {
                    table_name,
                    index_name: (*index_name).clone(),
                    first: first_idx,
                    second: second_idx,
                });
            }
        }
    }

    // Handle indexes for tables that exist in only one schema
    for table_key in index_tables1.difference(&index_tables2) {
        if let Some(indexes) = index_map1.get(*table_key) {
            for (_, index_list) in indexes {
                for index in index_list {
                    indexes_only_in_first.push(index.clone());
                }
            }
        }
    }

    for table_key in index_tables2.difference(&index_tables1) {
        if let Some(indexes) = index_map2.get(*table_key) {
            for (_, index_list) in indexes {
                for index in index_list {
                    indexes_only_in_second.push(index.clone());
                }
            }
        }
    }

    IndexDiff{
        indexes_only_in_first,
        indexes_only_in_second,
        indexes_with_different_definitions,
    }

}

pub fn generate_sql_index_diff(diff: &IndexDiff) -> HashMap<String, String> {
    let mut sql_statements = HashMap::new();
    // Handle indexes - Drop indexes that exist only in schema 1
    if !diff.indexes_only_in_first.is_empty() {
        for index in &diff.indexes_only_in_first {
            let key = format!("Drop index in Schema 1 (Schema 1 only)");
            let value = format!(
                "DROP INDEX `{}` ON `{}`.{};",
                index.index_name, index.table_schema, index.table_name
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Handle indexes - Add indexes that exist only in schema 2
    if !diff.indexes_only_in_second.is_empty() {
        for index in &diff.indexes_only_in_second {
            let key = format!("Add index in Schema 1 (Schema 2 only)");
            let is_unique = matches!(index.non_unique, Some(0));
            let value = if is_unique {
                format!(
                    "CREATE UNIQUE INDEX `{}` ON `{}`.{} ({});",
                    index.index_name, index.table_schema, index.table_name, index.column_name
                )
            } else {
                format!(
                    "CREATE INDEX `{}` ON `{}`.{} ({});",
                    index.index_name, index.table_schema, index.table_name, index.column_name
                )
            };
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Handle indexes - Modify indexes that have different definitions
    if !diff.indexes_with_different_definitions.is_empty() {
        for diff_item in &diff.indexes_with_different_definitions {
            let key = format!("Modify index in Schema 1");
            // For now, drop and recreate the index with new definition
            let drop_value = format!(
                "DROP INDEX `{}` ON `{}`.{};",
                diff_item.first.index_name, diff_item.first.table_schema, diff_item.table_name
            );
            let is_unique = matches!(diff_item.second.non_unique, Some(0));
            let create_value = if is_unique {
                format!(
                    "CREATE UNIQUE INDEX `{}` ON `{}`.{} ({});",
                    diff_item.second.index_name, diff_item.second.table_schema, diff_item.second.table_name, diff_item.second.column_name
                )
            } else {
                format!(
                    "CREATE INDEX `{}` ON `{}`.{} ({});",
                    diff_item.second.index_name, diff_item.second.table_schema, diff_item.second.table_name, diff_item.second.column_name
                )
            };
            let value = format!("{}\n{}", drop_value, create_value);
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Handle indexes - Add indexes that exist only in schema 1 (for schema 2)
    if !diff.indexes_only_in_first.is_empty() {
        for index in &diff.indexes_only_in_first {
            let key = format!("Add index in Schema 2 (Schema 1 only)");
            let is_unique = matches!(index.non_unique, Some(0));
            let value = if is_unique {
                format!(
                    "CREATE UNIQUE INDEX `{}` ON `{}`.{} ({});",
                    index.index_name, index.table_schema, index.table_name, index.column_name
                )
            } else {
                format!(
                    "CREATE INDEX `{}` ON `{}`.{} ({});",
                    index.index_name, index.table_schema, index.table_name, index.column_name
                )
            };
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Handle indexes - Drop indexes that exist only in schema 2
    if !diff.indexes_only_in_second.is_empty() {
        for index in &diff.indexes_only_in_second {
            let key = format!("Drop index in Schema 2 (Schema 2 only)");
            let value = format!(
                "DROP INDEX `{}` ON `{}`.{};",
                index.index_name, index.table_schema, index.table_name
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Handle indexes - Modify indexes that have different definitions (for schema 2)
    if !diff.indexes_with_different_definitions.is_empty() {
        for diff_item in &diff.indexes_with_different_definitions {
            let key = format!("Modify index in Schema 2");
            // For now, drop and recreate the index with new definition
            let drop_value = format!(
                "DROP INDEX `{}` ON `{}`.{};",
                diff_item.first.index_name, diff_item.first.table_schema, diff_item.first.table_name
            );
            let is_unique = matches!(diff_item.first.non_unique, Some(0));
            let create_value = if is_unique {
                format!(
                    "CREATE UNIQUE INDEX `{}` ON `{}`.{} ({});",
                    diff_item.first.index_name, diff_item.first.table_schema, diff_item.first.table_name, diff_item.first.column_name
                )
            } else {
                format!(
                    "CREATE INDEX `{}` ON `{}`.{} ({});",
                    diff_item.first.index_name, diff_item.first.table_schema, diff_item.first.table_name, diff_item.first.column_name
                )
            };
            let value = format!("{}\n{}", drop_value, create_value);
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    sql_statements
}


pub fn build_index_map(indexes: Vec<IndexInfo>) -> HashMap<String, IndexMap<String, Vec<IndexInfo>>> {
    let mut index_map: HashMap<String, IndexMap<String, Vec<IndexInfo>>> = HashMap::new();

    for index in indexes {
        // Use table_schema.table_name as the key to distinguish tables with the same name in different schemas
        let table_key = format!("`{}`.{}", index.table_schema, index.table_name);

        // Group indexes by table and index name
        let index_map_entry = index_map.entry(table_key).or_insert_with(IndexMap::new);
        let index_list = index_map_entry.entry(index.index_name.clone()).or_insert_with(Vec::new);
        index_list.push(index);
    }
    index_map
}

// Function to compare index lists for equality
fn compare_index_lists(list1: &Vec<IndexInfo>, list2: &Vec<IndexInfo>) -> bool {
    if list1.len() != list2.len() {
        return false;
    }

    // Sort both lists by column name and sequence to ensure consistent comparison
    let mut sorted_list1 = list1.clone();
    let mut sorted_list2 = list2.clone();

    sorted_list1.sort_by(|a, b| a.seq_in_index.cmp(&b.seq_in_index).then_with(|| a.column_name.cmp(&b.column_name)));
    sorted_list2.sort_by(|a, b| a.seq_in_index.cmp(&b.seq_in_index).then_with(|| a.column_name.cmp(&b.column_name)));

    // Compare each index in the sorted lists, ignoring seq_in_index for actual content comparison
    for (idx1, idx2) in sorted_list1.iter().zip(sorted_list2.iter()) {
        if idx1.table_schema != idx2.table_schema ||
           idx1.table_name != idx2.table_name ||
           idx1.index_name != idx2.index_name ||
           idx1.column_name != idx2.column_name ||
           idx1.collation != idx2.collation ||
           idx1.nullable != idx2.nullable ||
           idx1.index_type != idx2.index_type ||
           idx1.non_unique != idx2.non_unique {
            return false;
        }
    }

    true
}

pub fn create_index_info(json: &str) -> Result<Vec<IndexInfo>, String>{
     match serde_json::from_str::<Vec<IndexInfo>>(json) {
        Ok(columns) => return Ok(columns),
        Err(_) => {
            match serde_json::from_str::<Vec<MariaDBJson<IndexInfo>>>(json){
                Ok(column) => {
                        for i in 1..column.len(){
                            if column[i].data.is_some() {return Ok(column[i].data.clone().unwrap());}
                        }
                        return Err("Cannot find data".to_string());
                },
                Err(e) => return Err(format!("Error parsing first JSON: {}", e)),
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_builder_success() {
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
            .non_unique_opt(0)  // 0 for unique index
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
        assert_eq!(index_info.sub_part, None);
        assert_eq!(index_info.packed, None);
        assert_eq!(index_info.non_unique, Some(0));
    }

    #[test]
    fn test_builder_missing_required_fields() {
        // Test missing table_schema
        let result = IndexInfo::builder()
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("YES")
            .index_type("BTREE")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("table_schema is required"));

        // Test missing table_name
        let result = IndexInfo::builder()
            .table_schema("public")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("YES")
            .index_type("BTREE")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("table_name is required"));

        // Test missing index_name
        let result = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .column_name("email")
            .nullable("YES")
            .index_type("BTREE")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("index_name is required"));

        // Test missing column_name
        let result = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .nullable("YES")
            .index_type("BTREE")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("column_name is required"));

        // Test missing nullable
        let result = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .index_type("BTREE")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nullable is required"));

        // Test missing index_type
        let result = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("YES")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("index_type is required"));
    }

    #[test]
    fn test_builder_with_empty_string_fields() {
        let result = IndexInfo::builder()
            .table_schema("")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .nullable("YES")
            .index_type("BTREE")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("table_schema is required"));
    }

    #[test]
    fn test_serialization() {
        let index_info = IndexInfo::builder()
            .table_schema("public")
            .table_name("users")
            .index_name("idx_users_email")
            .column_name("email")
            .seq_in_index(1)
            .collation_opt("A".to_string())
            .cardinality_opt(100)
            .nullable("YES")
            .index_type("BTREE")
            .non_unique_opt(0)  // 0 for unique index
            .build()
            .expect("Build should succeed");

        let serialized = serde_json::to_string(&index_info).expect("Serialization should succeed");
        let expected_fields = ["table_schema", "table_name", "index_name", "column_name",
                              "seq_in_index", "collation", "cardinality", "nullable", "index_type", "non_unique"];

        for field in expected_fields {
            assert!(serialized.contains(field), "Serialized JSON should contain field: {}", field);
        }
    }

    #[test]
    fn test_deserialization() {
        let json_data = r#"
        {
            "table_schema": "public",
            "table_name": "users",
            "index_name": "idx_users_email",
            "column_name": "email",
            "seq_in_index": 1,
            "collation": "A",
            "cardinality": 100,
            "sub_part": null,
            "packed": null,
            "nullable": "YES",
            "index_type": "BTREE",
            "non_unique": 0
        }
        "#;

        let index_info: IndexInfo = serde_json::from_str(json_data).expect("Deserialization should succeed");
        assert_eq!(index_info.table_schema, "public");
        assert_eq!(index_info.table_name, "users");
        assert_eq!(index_info.index_name, "idx_users_email");
        assert_eq!(index_info.column_name, "email");
        assert_eq!(index_info.seq_in_index, Some(1));
        assert_eq!(index_info.collation, Some("A".to_string()));
        assert_eq!(index_info.cardinality, Some(100));
        assert_eq!(index_info.sub_part, None);
        assert_eq!(index_info.packed, None);
        assert_eq!(index_info.nullable, "YES");
        assert_eq!(index_info.index_type, "BTREE");
        assert_eq!(index_info.non_unique, Some(0));
    }

    #[test]
    fn test_deserialization_with_optional_fields_missing() {
        let json_data = r#"
        {
            "table_schema": "public",
            "table_name": "users",
            "index_name": "idx_users_email",
            "column_name": "email",
            "nullable": "YES",
            "index_type": "BTREE"
        }
        "#;

        let index_info: IndexInfo = serde_json::from_str(json_data).expect("Deserialization should succeed");
        assert_eq!(index_info.table_schema, "public");
        assert_eq!(index_info.table_name, "users");
        assert_eq!(index_info.index_name, "idx_users_email");
        assert_eq!(index_info.column_name, "email");
        assert_eq!(index_info.nullable, "YES");
        assert_eq!(index_info.index_type, "BTREE");
        assert_eq!(index_info.seq_in_index, None);
        assert_eq!(index_info.collation, None);
        assert_eq!(index_info.cardinality, None);
        assert_eq!(index_info.sub_part, None);
        assert_eq!(index_info.packed, None);
        assert_eq!(index_info.non_unique, None);
    }

    #[test]
    fn test_deserialization_missing_required_field() {
        let json_data = r#"
        {
            "table_name": "users",
            "index_name": "idx_users_email",
            "column_name": "email",
            "nullable": "YES",
            "index_type": "BTREE"
        }
        "#;

        let result: Result<IndexInfo, _> = serde_json::from_str(json_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialization_with_empty_required_field() {
        let json_data = r#"
        {
            "table_schema": "",
            "table_name": "users",
            "index_name": "idx_users_email",
            "column_name": "email",
            "nullable": "YES",
            "index_type": "BTREE"
        }
        "#;

        // Deserialization should succeed, but building should fail
        let index_info: IndexInfo = serde_json::from_str(json_data).expect("Deserialization should succeed");
        let result = index_info.build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("table_schema is required"));
    }
}
