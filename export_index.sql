SELECT
     table_schema,
     table_name,
     index_name,
     column_name,
     seq_in_index,
     collation,
     cardinality,
     sub_part,
     packed,
     nullable,
     index_type,
     non_unique
FROM information_schema.STATISTICS
WHERE TABLE_SCHEMA NOT IN ('information_schema', 'performance_schema', 'mysql')
ORDER BY TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX;
