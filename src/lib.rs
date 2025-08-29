use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use wasm_bindgen::prelude::*;
use std::collections::{HashMap, HashSet};


// Custom deserializer to handle both string and integer representations
fn deserialize_optional_string_as_int<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
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


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnInfo {
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: String,
    pub column_default: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string_as_int")]
    pub character_maximum_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MariaDBJson {
    pub r#type: String,
    pub version: Option<String>,
    pub comment: Option<String>,
    pub name: Option<String>,
    pub database: Option<String>,
    pub data: Option<Vec<ColumnInfo>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaDiff {
    pub tables_only_in_first: Vec<(String, String)>, // (schema, table_name)
    pub tables_only_in_second: Vec<(String, String)>, // (schema, table_name)
    pub columns_only_in_first: Vec<ColumnInfo>,
    pub columns_only_in_second: Vec<ColumnInfo>,
    pub columns_with_different_definitions: Vec<ColumnDifference>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnDifference {
    pub table_name: String,
    pub column_name: String,
    pub first: ColumnInfo,
    pub second: ColumnInfo,
}

#[wasm_bindgen]
pub fn compare_schemas(json1: &str, json2: &str) -> String {
    // Parse the JSON strings into vectors of ColumnInfo
    let columns1 = match create_column_info(json1) {
        Ok(column) => column,
        Err(e) => return e,
    };

    let columns2 =  match create_column_info(json2) {
        Ok(column) => column,
        Err(e) => return e,
    };

    // Convert to maps for easier comparison
    let map1 = build_schema_map(columns1);
    let map2 = build_schema_map(columns2);

    // Perform the comparison
    let diff = compare_schema_maps(&map1, &map2);

    // Generate SQL diff statements
    let sql_statements = generate_sql_diff(&diff);

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

pub fn create_column_info(json: &str) -> Result<Vec<ColumnInfo>, String>{
     match serde_json::from_str::<Vec<ColumnInfo>>(json) {
        Ok(columns) => return Ok(columns),
        Err(_) => {
            match serde_json::from_str::<Vec<MariaDBJson>>(json){
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

pub fn build_schema_map(columns: Vec<ColumnInfo>) -> HashMap<String, HashMap<String, ColumnInfo>> {
    let mut schema_map: HashMap<String, HashMap<String, ColumnInfo>> = HashMap::new();

    for column in columns {
        // Use table_schema.table_name as the key to distinguish tables with the same name in different schemas
        let table_key = format!("{}.{}", column.table_schema, column.table_name);
        let column_name = column.column_name.clone();

        schema_map
            .entry(table_key)
            .or_insert_with(HashMap::new)
            .insert(column_name, column);
    }

    schema_map
}

pub fn compare_schema_maps(
    map1: &HashMap<String, HashMap<String, ColumnInfo>>,
    map2: &HashMap<String, HashMap<String, ColumnInfo>>
) -> SchemaDiff {
    let tables1: HashSet<&String> = map1.keys().collect();
    let tables2: HashSet<&String> = map2.keys().collect();

    // Find tables only in first schema
    let tables_only_in_first: Vec<String> = tables1.difference(&tables2)
        .map(|s| s.to_string())
        .collect();

    // Find tables only in second schema
    let tables_only_in_second: Vec<String> = tables2.difference(&tables1)
        .map(|s| s.to_string())
        .collect();

    // Find common tables
    let common_tables: HashSet<&String> = tables1.intersection(&tables2).cloned().collect();

    let mut columns_only_in_first: Vec<ColumnInfo> = Vec::new();
    let mut columns_only_in_second: Vec<ColumnInfo> = Vec::new();
    let mut columns_with_different_definitions: Vec<ColumnDifference> = Vec::new();

    // Compare columns in common tables
    for table_key in common_tables {
        let columns1 = &map1[table_key];
        let columns2 = &map2[table_key];

        let column_names1: HashSet<&String> = columns1.keys().collect();
        let column_names2: HashSet<&String> = columns2.keys().collect();

        // Columns only in first schema
        for col_name in column_names1.difference(&column_names2) {
            columns_only_in_first.push(columns1[*col_name].clone());
        }

        // Columns only in second schema
        for col_name in column_names2.difference(&column_names1) {
            columns_only_in_second.push(columns2[*col_name].clone());
        }

        // Compare common columns
        for col_name in column_names1.intersection(&column_names2) {
            let col1 = &columns1[*col_name];
            let col2 = &columns2[*col_name];

            if col1 != col2 {
                // Extract just the table name (without schema) for the ColumnDifference
                let table_name = table_key.split('.').nth(1).unwrap_or(table_key).to_string();

                columns_with_different_definitions.push(ColumnDifference {
                    table_name,
                    column_name: (*col_name).clone(),
                    first: col1.clone(),
                    second: col2.clone(),
                });
            }
        }
    }

    // For tables that exist only in one schema, we need to extract both schema and table name
    let tables_only_in_first_with_schema: Vec<(String, String)> = tables_only_in_first
        .iter()
        .map(|key| {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() >= 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("public".to_string(), parts[0].to_string()) // Default to public schema
            }
        })
        .collect();

    let tables_only_in_second_with_schema: Vec<(String, String)> = tables_only_in_second
        .iter()
        .map(|key| {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() >= 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("public".to_string(), parts[0].to_string()) // Default to public schema
            }
        })
        .collect();

    SchemaDiff {
        tables_only_in_first: tables_only_in_first_with_schema,
        tables_only_in_second: tables_only_in_second_with_schema,
        columns_only_in_first,
        columns_only_in_second,
        columns_with_different_definitions,
    }
}

    pub fn generate_sql_diff(diff: &SchemaDiff) -> HashMap<String, String> {
    let mut sql_statements = HashMap::new();

    // Changes needed to transform schema 1 into schema 2
    // Drop tables that exist only in schema 1
    if !diff.tables_only_in_first.is_empty() {
        for (table_schema, table_name) in &diff.tables_only_in_first {
            let key = format!("Drop table in Schema 1 (Schema 1 only)");
            let value = format!("DROP TABLE {}.{};", table_schema, table_name);
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Create tables that exist only in schema 2
    if !diff.tables_only_in_second.is_empty() {
        for (table_schema, table_name) in &diff.tables_only_in_second {
            let key = format!("Create table in Schema 1 (Schema 2 only)");
            let value = format!("-- CREATE TABLE {}.{} (...);", table_schema, table_name);
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Drop columns that exist only in schema 1
    if !diff.columns_only_in_first.is_empty() {
        for column in &diff.columns_only_in_first {
            let key = format!("Drop column in Schema 1 (Schema 1 only)");
            let value = format!(
                "ALTER TABLE {}.{} DROP COLUMN {};",
                column.table_schema, column.table_name, column.column_name
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Add columns that exist only in schema 2
    if !diff.columns_only_in_second.is_empty() {
        for column in &diff.columns_only_in_second {
            let column_def = format_column_definition(column);
            let key = format!("Add column in Schema 1 (Schema 2 only)");
            let value = format!(
                "ALTER TABLE {}.{} ADD COLUMN {} {};",
                column.table_schema, column.table_name, column.column_name, column_def
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Modify columns that have different definitions (to match schema 2)
    if !diff.columns_with_different_definitions.is_empty() {
        for diff_item in &diff.columns_with_different_definitions {
            // Generate a MODIFY statement based on the second schema (target)
            let column_def = format_column_definition(&diff_item.second);
            let key = format!("Modify column in Schema 1");
            let value = format!(
                "ALTER TABLE {}.{} MODIFY COLUMN {} {};",
                diff_item.second.table_schema, diff_item.table_name, diff_item.column_name, column_def
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Changes needed to transform schema 2 into schema 1 (reverse of above)
    // Drop tables that exist only in schema 2
    if !diff.tables_only_in_second.is_empty() {
        for (table_schema, table_name) in &diff.tables_only_in_second {
            let key = format!("Drop table in Schema 2 (Schema 2 only)");
            let value = format!("DROP TABLE {}.{};", table_schema, table_name);
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Create tables that exist only in schema 1
    if !diff.tables_only_in_first.is_empty() {
        for (table_schema, table_name) in &diff.tables_only_in_first {
            let key = format!("Create table in Schema 2 (Schema 1 only)");
            let value = format!("-- CREATE TABLE {}.{} (...);", table_schema, table_name);
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Add columns that exist only in schema 1 (were dropped from schema 2)
    if !diff.columns_only_in_first.is_empty() {
        for column in &diff.columns_only_in_first {
            let column_def = format_column_definition(column);
            let key = format!("Add column in Schema 2 (Schema 1 only)");
            let value = format!(
                "ALTER TABLE {}.{} ADD COLUMN {} {};",
                column.table_schema, column.table_name, column.column_name, column_def
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Drop columns that exist only in schema 2
    if !diff.columns_only_in_second.is_empty() {
        for column in &diff.columns_only_in_second {
            let key = format!("Drop column in Schema 2 (Schema 2 only)");
            let value = format!(
                "ALTER TABLE {}.{} DROP COLUMN {};",
                column.table_schema, column.table_name, column.column_name
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    // Modify columns that have different definitions (to match schema 1)
    if !diff.columns_with_different_definitions.is_empty() {
        for diff_item in &diff.columns_with_different_definitions {
            // Generate a MODIFY statement based on the first schema (target)
            let column_def = format_column_definition(&diff_item.first);
            let key = format!("Modify column in Schema 2");
            let value = format!(
                "ALTER TABLE {}.{} MODIFY COLUMN {} {};",
                diff_item.first.table_schema, diff_item.table_name, diff_item.column_name, column_def
            );
            if let Some(existing) = sql_statements.get_mut(&key) {
                *existing = format!("{}\n{}", existing, value);
            } else {
                sql_statements.insert(key, value);
            }
        }
    }

    sql_statements
    }
//                     column.table_schema, column.table_name, column.column_name
//                 )
//             );
//         }
//     }
//
//     // Modify columns that have different definitions (to match schema 1)
//     if !diff.columns_with_different_definitions.is_empty() {
//         for diff_item in &diff.columns_with_different_definitions {
//             // Generate a MODIFY statement based on the first schema (target)
//             let column_def = format_column_definition(&diff_item.first);
//             sql_statements.insert(
//                 format!("Modify column in Schema 2"),
//                 format!(
//                     "ALTER TABLE {}.{} MODIFY COLUMN {} {};",
//                     diff_item.first.table_schema, diff_item.table_name, diff_item.column_name, column_def
//                 )
//             );
//         }
//     }
//
//     sql_statements
// }

fn format_column_definition(column: &ColumnInfo) -> String {
    let mut definition = column.data_type.clone();

    // Add character maximum length for string types
    if let Some(length) = column.character_maximum_length {
        if column.data_type == "varchar" || column.data_type == "char" {
            definition = format!("{}({})", column.data_type, length);
        }
    }

    // Add NULL/NOT NULL constraint
    if column.is_nullable == "NO" {
        definition.push_str(" NOT NULL");
    } else {
        definition.push_str(" NULL");
    }

    // Add default value if present
    if let Some(default) = &column.column_default {
        if default == "NULL" {
            definition.push_str(" DEFAULT NULL");
        } else if default.chars().all(|c| c.is_ascii_digit()) {
            definition.push_str(&format!(" DEFAULT {}", default));
        } else {
            definition.push_str(&format!(" DEFAULT '{}'", default.replace("'", "''")));
        }
    }

    definition
}
