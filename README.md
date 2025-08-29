# Database Schema Comparison Tool

This tool compares two database schemas exported from `information_schema.COLUMNS` and highlights the differences between them.

## How to Use

1. Export schema information from your databases using a query like:
   ```sql
   SELECT table_name, column_name, data_type, is_nullable, column_default, character_maximum_length
   FROM information_schema.COLUMNS
   WHERE table_schema = 'your_database_name';
   ```
   
2. Convert the results to JSON format (array of objects).

3. Paste the JSON from each database into the respective text areas in the web interface.

4. Click "Compare Schemas" to see the differences.

## Building and Running

### Prerequisites
- Rust toolchain (https://rustup.rs/)
- wasm-pack (will be installed automatically by build.sh)

### Building
```bash
./build.sh
```

### Running
Open `index.html` in a web browser.

## Features
- Compares tables present in each schema
- Compares columns in common tables
- Highlights columns with different definitions
- Shows detailed differences in column properties

## Example JSON Format
```json
[
  {
    "table_name": "users",
    "column_name": "id",
    "data_type": "integer",
    "is_nullable": "NO",
    "column_default": null,
    "character_maximum_length": null
  },
  {
    "table_name": "users",
    "column_name": "name",
    "data_type": "varchar",
    "is_nullable": "YES",
    "column_default": null,
    "character_maximum_length": 255
  }
]
```