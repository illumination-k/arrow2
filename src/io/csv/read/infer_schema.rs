use std::{
    collections::HashSet,
    io::{Read, Seek},
};

use crate::datatypes::{DataType, Schema};
use crate::error::Result;

use super::super::utils::merge_schema;
use super::{ByteRecord, Reader};

/// Infers a [`Schema`] of a CSV file by reading through the first n records up to `max_rows`.
/// Seeks back to the begining of the file _after_ the header
pub fn infer_schema<R: Read + Seek, F: Fn(&[u8]) -> DataType>(
    reader: &mut Reader<R>,
    max_rows: Option<usize>,
    has_header: bool,
    infer: &F,
) -> Result<Schema> {
    // get or create header names
    // when has_header is false, creates default column names with column_ prefix
    let headers: Vec<String> = if has_header {
        reader.headers()?.iter().map(|s| s.to_string()).collect()
    } else {
        let first_record_count = &reader.headers()?.len();
        (0..*first_record_count)
            .map(|i| format!("column_{}", i + 1))
            .collect()
    };

    // save the csv reader position after reading headers
    let position = reader.position().clone();

    let header_length = headers.len();
    // keep track of inferred field types
    let mut column_types: Vec<HashSet<DataType>> = vec![HashSet::new(); header_length];

    let mut records_count = 0;

    let mut record = ByteRecord::new();
    let max_records = max_rows.unwrap_or(usize::MAX);
    while records_count < max_records {
        if !reader.read_byte_record(&mut record)? {
            break;
        }
        records_count += 1;

        for (i, column) in column_types.iter_mut().enumerate() {
            if let Some(string) = record.get(i) {
                column.insert(infer(string));
            }
        }
    }

    let fields = merge_schema(&headers, &mut column_types);

    // return the reader seek back to the start
    reader.seek(position)?;

    Ok(Schema::new(fields))
}
