use std;

use chrono;
use csv;
use encoding;
use failure;
use ipnet;
use url;

use types;

type StringRecord = (String, String, String, String, String, String);

pub struct Reader<StreamReader>
where
    StreamReader: std::io::BufRead,
{
    updated: types::DateTime,
    csv_reader: csv::Reader<StreamReader>,
}

impl<StreamReader> Reader<StreamReader>
where
    StreamReader: std::io::BufRead,
{
    fn parse_update_datetime(reader: &mut StreamReader) -> Result<types::DateTime, failure::Error> {
        let mut first_line = String::new();
        let _ = reader.read_line(&mut first_line)?;

        let space_pos = first_line.find(':').ok_or_else(|| {
            format_err!(
                "No ':' in first line: \"{}\" (should be in format \"Updated: $DATE_TIME\")",
                first_line
            )
        })?;
        let (_, updated) = first_line.split_at(space_pos + 1);
        let updated = updated.trim();

        let updated = chrono::DateTime::parse_from_str(updated, "%Y-%m-%d %H:%M:%S %z")
            .map_err(|error| {
                format_err!(
                    "Invalid date and time in first line: \"{}\" (\"{}\": {})",
                    first_line,
                    updated,
                    error
                )
            })?;
        Ok(updated.naive_utc())
    }

    /// Parse data from buffered reader.
    pub fn from_buf_reader(mut reader: StreamReader) -> Result<Self, failure::Error> {
        Ok(Self {
            updated: Self::parse_update_datetime(&mut reader)?,
            csv_reader: csv::Reader::from_reader(reader)
                .delimiter(b';')
                .has_headers(false)
                .flexible(true),
        })
    }

    /// Parse data from normal (not buffered) reader.
    pub fn from_reader<UnbufferedReader: std::io::Read>(
        reader: UnbufferedReader,
    ) -> Result<Reader<std::io::BufReader<UnbufferedReader>>, failure::Error> {
        let buf_reader = std::io::BufReader::new(reader);
        Reader::<std::io::BufReader<UnbufferedReader>>::from_buf_reader(buf_reader)
    }

    /// Parse data from file specified by path.
    pub fn from_file<Path: AsRef<std::path::Path>>(
        path: Path,
    ) -> Result<Reader<std::io::BufReader<std::fs::File>>, failure::Error> {
        Self::from_reader(std::fs::File::open(path)?)
    }

    /// Date of last update of this list.
    pub fn get_timestamp(&self) -> &types::DateTime {
        &self.updated
    }

    pub fn records(&mut self) -> Records<StreamReader> {
        Records {
            csv_records: self.csv_reader.byte_records(),
        }
    }
}

pub struct Records<'a, StreamReader: 'a>
where
    StreamReader: std::io::BufRead,
{
    csv_records: csv::ByteRecords<'a, StreamReader>,
}

impl<'a, StreamReader: 'a> Records<'a, StreamReader>
where
    StreamReader: std::io::BufRead,
{
    fn str_from_cp1251(raw: &[u8]) -> Result<String, failure::Error> {
        use encoding::Encoding;

        encoding::all::WINDOWS_1251
            .decode(raw, encoding::DecoderTrap::Strict)
            .map_err(|error| format_err!("Invalid CP1251 string ({})", error))
    }

    fn str_rec_from_cp1251(raw_vec: &[Vec<u8>]) -> Result<StringRecord, failure::Error> {
        Ok((
            Self::str_from_cp1251(&raw_vec[0])?,
            Self::str_from_cp1251(&raw_vec[1])?,
            Self::str_from_cp1251(&raw_vec[2])?,
            Self::str_from_cp1251(&raw_vec[3])?,
            Self::str_from_cp1251(&raw_vec[4])?,
            Self::str_from_cp1251(&raw_vec[5])?,
        ))
    }

    fn parse_for_each<ParseFn>(addr_str: &str, delim: &str, mut func: ParseFn) -> Result<(), failure::Error>
    where
        ParseFn: FnMut(&str) -> Result<(), failure::Error>,
    {
        for part in addr_str.split(delim) {
            let part = part.trim();
            if !part.is_empty() {
                func(part)?;
            }
        }

        Ok(())
    }

    fn parse_ipv4_addresses(addr_str: &str, addresses: &mut types::Addresses) -> Result<(), failure::Error> {
        use std::str::FromStr;

        Self::parse_for_each(addr_str, "|", |part| {
            if part.contains('/') {
                let _ = addresses.insert(types::Address::IPv4Network(ipnet::Ipv4Net::from_str(part)?));
            } else {
                let _ = addresses.insert(types::Address::IPv4(std::net::Ipv4Addr::from_str(part)?));
            }

            Ok(())
        })
    }

    fn parse_domain_name(addr_str: &str, addresses: &mut types::Addresses) -> Result<(), failure::Error> {
        Self::parse_for_each(addr_str, "|", |part| {
            {
                if part.starts_with('*') {
                    let _ = addresses.insert(types::Address::WildcardDomainName(part.into()));
                } else {
                    let _ = addresses.insert(types::Address::DomainName(part.into()));
                }
            }

            Ok(())
        })
    }

    fn parse_url(addr_str: &str, addresses: &mut types::Addresses) -> Result<(), failure::Error> {
        use std::str::FromStr;

        // We are using " | " as a delimiter because URL itself may contain '|'.
        Self::parse_for_each(addr_str, " | ", |part| {
            {
                let _ = addresses.insert(types::Address::URL(url::Url::from_str(part)?));
            }

            Ok(())
        })
    }

    fn parse_order_date(date_str: &str) -> Result<types::Date, failure::Error> {
        Ok(types::Date::parse_from_str(date_str.trim(), "%Y-%m-%d")?)
    }

    fn parse_record(record: &StringRecord) -> Result<types::Record, failure::Error> {
        let mut addresses = types::Addresses::new();

        Self::parse_ipv4_addresses(&record.0, &mut addresses)?;
        Self::parse_domain_name(&record.1, &mut addresses)?;
        Self::parse_url(&record.2, &mut addresses)?;

        Ok(types::Record {
            addresses,
            organization: record.3.trim().into(),
            order_id: record.4.trim().into(),
            order_date: Self::parse_order_date(&record.5)?,
        })
    }
}

impl<'a, StreamReader: 'a> Iterator for Records<'a, StreamReader>
where
    StreamReader: std::io::BufRead,
{
    type Item = Result<types::Record, failure::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.csv_records.next().map(|csv_result| -> Self::Item {
            csv_result
                .map_err(|csv_err| csv_err.into())
                .and_then(|raw_record| Self::str_rec_from_cp1251(&raw_record))
                .and_then(|str_record| Self::parse_record(&str_record))
        })
    }
}
