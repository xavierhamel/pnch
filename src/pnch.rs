use std::{str, fmt::Write};
use crate::{storage, time, tag, error::GlobalError};
use colored::*;

/// A pnch is an activity.
///
/// It is represented with a beginning (in), an end (out), a tag which helps categorize the
/// activity and a description which differentiate between each activity with a same tag.
#[derive(Debug, PartialEq, Eq)]
pub struct Pnch {
    /// The id of the entry
    pub id: u32,
    /// The date for the activity.
    pub date: time::Date,
    /// The time when the activity starts.
    pub _in: time::Time,
    /// The time when the activity ends.
    pub out: Option<time::Time>,
    /// The tag.
    pub tag: Option<tag::Tag>,
    /// The description of the activity.
    pub description: Option<String>,
}

impl Pnch {
    /// size of the date field in bytes
    const DATE_SIZE: usize = time::Date::SIZE;
    /// size of the in field in bytes
    const IN_SIZE: usize = time::Time::SIZE;
    /// size of the out field in bytes
    const OUT_SIZE: usize = time::Time::SIZE;
    /// size of the tag id field in bytes
    const TAG_ID_SIZE: usize = tag::Tag::ID_SIZE;
    /// size of the description field in bytes
    const DESCRIPTION_SIZE: usize = 80;
    /// total size of a pnch when saved in a file in bytes.
    const SIZE: usize = Self::DATE_SIZE + Self::TAG_ID_SIZE +  Self::OUT_SIZE + Self::IN_SIZE + Self::DESCRIPTION_SIZE;

    pub fn new(id: u32, time: time::Time, tag: Option<tag::Tag>, description: Option<String>) -> Self {
        Self {
            id,
            _in: time,
            out: None,
            date: time::Date::today(),
            description,
            tag,
        }
    }

    pub fn out(&mut self, time: time::Time, tag: Option<tag::Tag>, description: Option<String>) -> Result<(), GlobalError> {
        if self.out.is_some() {
            return Err(GlobalError::pnch_already_closed());
        }
        if time < self._in {
            return Err(GlobalError::pnch_out_before_in(self._in, time));
        }
        if let Some(desc) = description {
            if self.description.is_some() {
                return Err(GlobalError::desc_already_specified(
                    &tag.map(|t| t.to_string()).unwrap_or(String::new()),
                    &desc)
                );
            }
            self.description = Some(desc);
            self.tag = tag;
        }
        if self.description.is_none() {
            return Err(GlobalError::desc_not_specified());
        }
        self.out = Some(time);
        Ok(())
    }

    fn try_from(id: u32, chunk: &[u8], tags: &tag::Tags) -> Result<Self, GlobalError> {
        if chunk.len() != Self::SIZE {
            return Err(GlobalError::wrong_byte_len("pnch", chunk.len(), Self::SIZE));
        }
        let (date_bytes, chunk) = chunk.split_at(Self::DATE_SIZE);
        let (in_bytes, chunk) = chunk.split_at(Self::IN_SIZE);
        let (out_bytes, chunk) = chunk.split_at(Self::OUT_SIZE);
        let (tag_id_bytes, description_bytes) = chunk.split_at(Self::TAG_ID_SIZE);
        let description_bytes = description_bytes
            .iter()
            .copied()
            .filter(|&c| c != 0)
            .collect::<Vec<u8>>();

        let tag_id_bytes = tag_id_bytes
            .try_into()
            .expect("split_at panics if not correct size");
        let tag = match u32::from_le_bytes(tag_id_bytes) {
            0xFFFF => None,
            tag_id @ _ => tags.get(tag_id)
        };
        let out = match out_bytes {
            &[0xFF, 0xFF] => None,
            bytes @ _ => Some(bytes.try_into()?),
        };
        let description = match description_bytes.len() {
            0 => None,
            _ => Some(String::from_utf8(description_bytes)?),
        };
        Ok(Pnch {
            id,
            date: date_bytes.try_into()?,
            _in: in_bytes.try_into()?,
            out,
            tag,
            description
        })
    }

    pub fn duration(&self) -> Option<time::Duration> {
        self.out.map(|out| out - self._in)
    }
}

impl From<&Pnch> for Vec<u8> {
    fn from(pnch: &Pnch) -> Self {
        let mut buffer = Vec::with_capacity(Pnch::SIZE);
        buffer.extend_from_slice(&pnch.date.to_le_bytes());
        buffer.extend_from_slice(&pnch._in.to_le_bytes());

        let out_bytes = pnch.out.unwrap_or(time::Time::none()).to_le_bytes();
        buffer.extend_from_slice(&out_bytes);

        let tag_id_bytes = match &pnch.tag {
            Some(tag) => tag.id.to_le_bytes(),
            None => tag::Tag::none().id.to_le_bytes()
        };
        buffer.extend_from_slice(&tag_id_bytes);
        if let Some(description) = &pnch.description {
            buffer.extend_from_slice(description.as_bytes());
        }
        buffer.append(&mut vec![0; Pnch::SIZE - buffer.len()]);
        buffer
    }
}

impl std::fmt::Display for Pnch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  #{} >", self.id)?;
        match self.out {
            Some(out) => writeln!(f, " From {} to {out} ({})",
                self._in, out - self._in)?,
            None => writeln!(f, " Since {} ", self._in)?,
        }
        match &self.tag {
            Some(tag) => write!(f, "    {tag} ")?,
            _ => write!(f, "    [---] ")?,
        }
        match &self.description {
            Some(description) => write!(f, "{description}")?,
            _ => write!(f, "no description")?,
        }
        Ok(())
    }
}

impl std::cmp::Ord for Pnch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.date, self._in).cmp(&(&other.date, other._in))
    }
}

impl std::cmp::PartialOrd for Pnch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (&self.date, self._in).partial_cmp(&(&other.date, other._in))
    }
}

#[derive(Debug, Clone)]
pub struct Description {
    pub tag: Option<String>,
    pub description: String,
}

impl std::str::FromStr for Description {
    type Err = GlobalError;
    fn from_str(str: &str) -> Result<Self, GlobalError> {
        if let Some((tag, description)) = str.split_once("/") {
            Ok(Self {
                tag: Some(tag.to_owned()),
                description: description.to_owned(),
            })
        } else {
            Ok(Self {
                tag: None,
                description: str.to_owned(),
            })
        }
    }
}

/// A group of pnch.
pub struct Pnchs(pub Vec<Pnch>);

impl Pnchs {
    const PNCHS_FILE_NAME: &'static str = "pnchs.db";

    pub fn load(tags: &tag::Tags) -> Result<Self, GlobalError> {
        let buffer = storage::load(Self::PNCHS_FILE_NAME)?;
        let mut pnchs = buffer
            .chunks_exact(Pnch::SIZE)
            .into_iter()
            .enumerate()
            .map(|(id, chunk)| Pnch::try_from(id as u32, chunk, tags))
            .collect::<Result<Vec<Pnch>, GlobalError>>()?;
        pnchs.sort();
        Ok(Self(pnchs))
    }

    pub fn _in(&mut self, pnch: Pnch) -> Result<(), GlobalError> {
        match self.0.last() {
            Some(pnch) if pnch.out.is_none() => {
                return Err(GlobalError::pnch_already_open());
            }
            _ => {
                self.0.push(pnch);
            }
        }
        Ok(())
    }

    pub fn get(&mut self, id: u32) -> Option<&mut Pnch> {
        self.0.iter_mut().find(|pnch| pnch.id == id)
    }

    pub fn get_last(&mut self) -> Option<&mut Pnch> {
        self.0.last_mut()
    }

    pub fn save(&self) -> Result<(), GlobalError> {
        let path = storage::build_path(Self::PNCHS_FILE_NAME)?;
        let content = self.0
            .iter()
            .map(|pnch| Vec::from(pnch))
            .flatten()
            .collect::<Vec<u8>>();
        std::fs::write(path, content)
            .map_err(|_| GlobalError::fs("save", "pnchs"))?;
        Ok(())
    }

    pub fn into_csv(self) -> Result<String, GlobalError> {
        self.0
            .into_iter()
            .map(|pnch| {
                let mut line = String::new();
                match pnch.tag {
                    Some(tag) => write!(&mut line, "{},", tag.tag)?,
                    None => write!(&mut line, ",")?,
                }
                write!(&mut line, "{},", pnch.description.unwrap_or_default())?;
                write!(&mut line, "{},", pnch.date)?;
                write!(&mut line, "{},", pnch._in)?;
                match pnch.out {
                    Some(out) => write!(&mut line, "{out}\n")?,
                    None => write!(&mut line, "\n")?,
                }
                Ok(line)
            })
            .collect::<Result<String, std::fmt::Error>>()
            .map_err(|_| GlobalError::formatting("csv"))
    }

    pub fn into_table(self) -> PnchsTable {
        PnchsTable(self)
    }

    pub fn duration(&self) -> time::Duration {
        self.0
            .iter()
            .filter_map(|pnch| pnch.duration())
            .fold(time::Duration::zero(), |total, duration| {
                total + duration
            })
    }
}

impl std::fmt::Display for Pnchs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() == 0 {
            // TODO: The error should not be printed here
            // We should also add a HINT to clarify that the filter was
            // probably too strict.
            return writeln!(f, "{}\n    No pnchs were found.", "error:".red());
        }
        let total_duration = self.duration();
        writeln!(f, "You were punched in for {total_duration}")?;
        self.0
            .iter()
            .try_fold(time::Date::min(), |mut date, pnch| {
                if date != pnch.date {
                    date = pnch.date.clone();
                    writeln!(f, "\n{date}")?;
                }
                writeln!(f, "{pnch}")?;
                Ok(date)
            })?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Format {
    Table,
    List,
    Csv
}

impl str::FromStr for Format {
    type Err = GlobalError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match &value.to_lowercase()[..] {
            "table" => Ok(Self::Table),
            "list" => Ok(Self::List),
            "csv" => Ok(Self::Csv),
            _ => Err(GlobalError::parse("`pretty` or `csv`"))
        }
    }
}

pub struct PnchsTable(Pnchs);

impl PnchsTable {
    const COLS: usize = 6;
    const COLS_WIDTH: [usize; Self::COLS] = [12, 7, 16, 7, 7, 44];

    fn pnch_to_cells(&self, pnch: &Pnch, date: &mut time::Date) -> (bool, Vec<String>) {
        let mut cells = Vec::new();
        let did_date_update = if pnch.date != *date {
            *date = pnch.date.clone();
            cells.push(format!("{date}"));
            true
        } else {
            cells.push(String::new());
            false
        };
        cells.push(pnch.id.to_string());
        cells.push(pnch.tag.as_ref().map(|t| t.tag.to_string()).unwrap_or(String::from("---")));
        cells.push(pnch._in.to_string());
        cells.push(pnch.out.as_ref().map(|o| o.to_string()).unwrap_or(String::new()));
        cells.push(pnch.description.clone().unwrap_or(String::new()));
        (did_date_update, cells)
    }

    fn cells_to_string(&self, cells: Vec<String>) -> String {
        let mut cells = cells
            .iter()
            .enumerate()
            .map(|(idx, cell)| {
                format!("│ {:<width$} ", cell, width = Self::COLS_WIDTH[idx] - 2)
            })
            .collect::<String>();
        cells.push_str("│");
        cells
    }

    fn separator(&self, left: &str, mid: &str, right: &str) -> String {
        let mut separator = String::from(left);
        separator.push_str(&Self::COLS_WIDTH.iter().enumerate().map(|(idx, width)| {
            let mut end = mid;
            if idx == Self::COLS_WIDTH.len() - 1 {
                end = right
            }
            format!("{}{end}", &"-".repeat(*width))
        }).collect::<String>());
        separator
    }
}

//┌────────────┬───────┬────────────────┬───────┬───────┬────────────────────────────────────┐
//│ Date       │ Id    │ Tag            │ In    │ Out   │ Description                        │
//├────────────┼───────┼────────────────┼───────┼───────┼────────────────────────────────────┤
//│ 2023-05-10 │  9:23 │ RDG-123        │  9:23 │ 10:33 │ Ceci est ma description            │
//│            │  9:23 │ RDG-123        │  9:23 │ 10:33 │ Ceci est ma description            │
//│            │  9:23 │ RDG-123-123-2..│  9:23 │ 10:33 │ Ceci est ma description            │
//├────────────┼───────┼────────────────┼───────┼───────┼────────────────────────────────────┤
//│ 2023-05-11 │  9:23 │ RDG-123        │  9:23 │ 10:33 │ Ceci est ma description            │
//│            │  9:23 │ RDG-123        │  9:23 │ 10:33 │ Ceci est ma description            │
//│            │  9:23 │ RDG-123        │  9:23 │ 10:33 │ Ceci est ma description            │
//└────────────┴───────┴────────────────┴───────┴───────┴────────────────────────────────────┘
impl std::fmt::Display for PnchsTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.0.len() == 0 {
            // TODO: The error should not be printed here
            // We should also add a HINT to clarify that the filter was
            // probably too strict.
            return writeln!(f, "{}\n    No pnchs were found.", "error:".red());
        }
        let separator = self.separator("├", "┼", "┤");
        let mut rows = vec![
            self.separator("┌", "┬", "┐"),
            self.cells_to_string(vec![
                String::from("Date"), String::from("Id"), String::from("Tag"),
                String::from("In"), String::from("Out"), String::from("Description"),
            ])
        ];

        let mut date = time::Date::min();
        for pnch in self.0.0.iter() {
            let (did_date_update, cells) = self.pnch_to_cells(pnch, &mut date);
            if did_date_update {
                rows.push(separator.clone());
            }
            rows.push(self.cells_to_string(cells));
        }
        rows.push(self.separator("└", "┴", "┘"));
        let table = rows.join("\n");
        let total_duration = self.0.duration();
        writeln!(f, "You were punched in for {total_duration}")?;
        writeln!(f, "{table}")
    }
}
