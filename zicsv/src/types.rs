use std;

use chrono;
use csv;
use encoding;
use ipnet;

use url;
#[cfg(feature = "serialization")]
use url_serde;

#[cfg(feature = "serialization")]
use ipnet_serde;

/// Internet address blocked by Roskomnadzor.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum Address {
    /// Blocked by IPv4 address.
    IPv4(std::net::Ipv4Addr),
    /// Blocked by IPv4 subnet.
    #[cfg_attr(feature = "serialization", serde(with = "ipnet_serde"))]
    IPv4Network(ipnet::Ipv4Net),
    /// Blocked by domain name.
    DomainName(String),
    /// Blocked by URL.
    #[cfg_attr(feature = "serialization", serde(with = "url_serde"))]
    URL(url::Url),
}

pub type Addresses = std::collections::BTreeSet<Address>;
pub type Date = chrono::NaiveDate;

/// One record from CSV.
#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Record {
    /// Blocked addresses.
    pub addresses: Addresses,
    /// Name of organization that requested blocking.
    pub organization: String,
    /// ID of official order.
    pub order_id: String,
    /// Date of official order.
    pub order_date: Date,
}

pub type DateTime = chrono::NaiveDateTime;
pub type Records = std::collections::LinkedList<Record>;

/// List of blocks issued by Roskomnadzor.
#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct List {
    /// Date of last update of this list.
    pub updated: DateTime,
    /// List of records.
    pub records: Records,
}

type StringRecord = (String, String, String, String, String, String);

impl List {
    fn parse_update_datetime<Reader: std::io::BufRead>(reader: &mut Reader) -> DateTime {
        let mut first_line = String::new();
        reader.read_line(&mut first_line).unwrap();

        let space_pos = first_line.find(' ').unwrap();
        let (_, updated) = first_line.split_at(space_pos + 1);
        let updated = updated.trim();

        let updated = chrono::DateTime::parse_from_str(updated, "%Y-%m-%d %H:%M:%S %z").unwrap();
        updated.naive_utc()
    }

    fn str_from_cp1251(raw: &[u8]) -> String {
        use encoding::Encoding;

        encoding::all::WINDOWS_1251
            .decode(raw, encoding::DecoderTrap::Strict)
            .unwrap()
    }

    fn str_rec_from_cp1251(raw_vec: &[Vec<u8>]) -> StringRecord {
        (
            Self::str_from_cp1251(&raw_vec[0]),
            Self::str_from_cp1251(&raw_vec[1]),
            Self::str_from_cp1251(&raw_vec[2]),
            Self::str_from_cp1251(&raw_vec[3]),
            Self::str_from_cp1251(&raw_vec[4]),
            Self::str_from_cp1251(&raw_vec[5]),
        )
    }

    fn parse_for_each<ParseFn>(addr_str: &str, delim: &str, mut func: ParseFn) -> ()
    where
        ParseFn: FnMut(&str),
    {
        for part in addr_str.split(delim) {
            let part = part.trim();
            if !part.is_empty() {
                func(part);
            }
        }
    }

    fn parse_ipv4_addresses(addr_str: &str, addresses: &mut Addresses) -> () {
        use std::str::FromStr;

        Self::parse_for_each(addr_str, "|", |part| if part.contains('/') {
            addresses.insert(Address::IPv4Network(
                ipnet::Ipv4Net::from_str(part).unwrap(),
            ));
        } else {
            addresses.insert(Address::IPv4(std::net::Ipv4Addr::from_str(part).unwrap()));
        });
    }

    fn parse_domain_name(addr_str: &str, addresses: &mut Addresses) -> () {
        Self::parse_for_each(addr_str, "|", |part| {
            addresses.insert(Address::DomainName(part.into()));
        });
    }

    fn parse_url(addr_str: &str, addresses: &mut Addresses) -> () {
        use std::str::FromStr;

        // We are using " | " as a delimiter because URL itself may contain '|'.
        Self::parse_for_each(addr_str, " | ", |part| {
            addresses.insert(Address::URL(url::Url::from_str(part).unwrap()));
        });
    }

    fn parse_order_date(date_str: &str) -> Date {
        Date::parse_from_str(date_str.trim(), "%Y-%m-%d").unwrap()
    }

    fn parse_record(record: &StringRecord) -> Record {
        let mut addresses = Addresses::new();

        Self::parse_ipv4_addresses(&record.0, &mut addresses);
        Self::parse_domain_name(&record.1, &mut addresses);
        Self::parse_url(&record.2, &mut addresses);

        Record {
            addresses,
            organization: record.3.trim().into(),
            order_id: record.4.trim().into(),
            order_date: Self::parse_order_date(&record.5),
        }
    }

    fn parse_records<Reader: std::io::BufRead>(reader: Reader) -> Records {
        let mut csv = csv::Reader::from_reader(reader)
            .delimiter(b';')
            .has_headers(false)
            .flexible(true);

        let mut records = Records::new();
        let mut line_n = 1u64; // First line is used for timestamp.
        for record in csv.byte_records() {
            line_n += 1;

            let record = record.unwrap();

            if record.len() != 6 {
                panic!(
                    "Invalid number of fields in line {}: {} != 6",
                    line_n,
                    record.len()
                );
            }

            let record = Self::str_rec_from_cp1251(&record);
            records.push_back(Self::parse_record(&record));
        }

        records
    }

    /// Load data from buffered reader.
    pub fn load_from_buf_reader<Reader: std::io::BufRead>(mut reader: Reader) -> List {
        List {
            updated: Self::parse_update_datetime(&mut reader),
            records: Self::parse_records(reader),
        }
    }

    /// Load data from normal (not buffered) reader.
    pub fn load_from_reader<Reader: std::io::Read>(reader: Reader) -> List {
        Self::load_from_buf_reader(std::io::BufReader::new(reader))
    }

    /// Load data from file specified by path.
    pub fn load_from_file<Path: AsRef<std::path::Path>>(path: Path) -> List {
        Self::load_from_reader(std::fs::File::open(path).unwrap())
    }
}
