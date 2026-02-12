use std::error::Error;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TimetableData {
    pub headers: Vec<String>,
    pub time_slots: Vec<String>,
    pub grid: Vec<Vec<String>>,
}

impl TimetableData {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        
        let mut raw: Vec<Vec<String>> = Vec::new();
        for result in rdr.records() {
            let record = result?;
            raw.push(record.iter().map(|s| s.to_string()).collect());
        }
        
        if raw.is_empty() { return Err("CSV is empty".into()); }
        
        let headers = raw[0].iter().skip(1).cloned().collect();
        let mut time_slots = Vec::new();
        let mut grid = Vec::new();
        
        for row in raw.iter().skip(1) {
            if row.is_empty() { continue; }
            time_slots.push(row[0].clone());
            let row_data: Vec<String> = row.iter().skip(1).cloned().collect();
            grid.push(row_data);
        }
        
        Ok(Self { headers, time_slots, grid })
    }

    pub fn save(path: &Path, row: usize, col: usize, val: String) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?;
            
        let mut records: Vec<Vec<String>> = Vec::new();
        for res in rdr.records() {
            records.push(res?.iter().map(|s| s.to_string()).collect());
        }
        
        if row + 1 < records.len() && col + 1 < records[row+1].len() {
            records[row+1][col+1] = val;
        }
        
        let mut wtr = csv::Writer::from_path(path)?;
        for rec in records {
            wtr.write_record(&rec)?;
        }
        wtr.flush()?;
        Ok(())
    }
}