use super::*;
use chrono::NaiveDate;

pub(super) fn read_sample_text(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("Could not read file: {}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .with_context(|| format!("Could not read file: {}", path.display()))?;
    Ok(decode_bytes(&bytes))
}

pub(super) fn csv_reader(path: &Path, delimiter: char) -> Result<csv::Reader<BufReader<File>>> {
    let file =
        File::open(path).with_context(|| format!("Could not read file: {}", path.display()))?;
    Ok(ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(BufReader::new(file)))
}

pub(super) fn column_index(headers: &[String], column: &Option<String>) -> Option<usize> {
    let column = column.as_ref()?;
    headers.iter().position(|header| header == column)
}

pub(super) fn date_from_byte_record(record: &ByteRecord, index: usize) -> Option<NaiveDate> {
    record
        .get(index)
        .and_then(|cell| parse_date(&decode_bytes(cell)))
}

pub(super) fn decode_record(record: &ByteRecord) -> Vec<String> {
    record.iter().map(decode_bytes).collect()
}

fn decode_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        text.to_string()
    } else {
        let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
        cow.into_owned()
    }
}

pub(super) fn sniff_delimiter(content: &str) -> char {
    let candidates = [';', ',', '\t', '|'];
    let lines: Vec<&str> = content.lines().take(12).collect();
    let mut best = (';', 0usize);
    for delimiter in candidates {
        let score = lines
            .iter()
            .map(|line| line.matches(delimiter).count())
            .filter(|count| *count > 0)
            .sum::<usize>();
        if score > best.1 {
            best = (delimiter, score);
        }
    }
    best.0
}
