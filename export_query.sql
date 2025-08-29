SELECT
    table_schema,
    table_name,
    column_name,
    data_type,
    is_nullable,
    column_default,
    character_maximum_length
FROM information_schema.COLUMNS
WHERE table_schema = 'your_database_name'
ORDER BY table_name, ordinal_position;
