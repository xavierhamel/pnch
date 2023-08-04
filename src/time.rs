use std::{str, default};
use crate::error::{self, GlobalError};

/// A period is used to specify a duration of time in term of days, weeks, months or years.
///
/// A period can be created from a string in the format `[n] period[s]` where n is a number and
/// period is one of `day[s]`, `week[s]`, `month[s]` or `year[s]`. For example, `3 weeks`, `year`
/// and `56 months` are all periods.
#[derive(Debug, Clone)]
pub enum Period {
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl Period {
    /// Hint on how to format a period as a string.
    const FORMAT_HINT: &'static str =
        "`n <period>` where `n` is a number and `<period>` is one of `days`, `weeks`, `months` or `years`";

    /// Substract the period to the current date, returning the date n period ago.
    ///
    /// For example, a period of 2 weeks ago will return the date from today minus 2 weeks.
    pub fn to_date_since_today(&self) -> Date {
        let days = match self {
            Self::Days(days) => *days,
            Self::Weeks(weeks) => weeks * 7,
            Self::Months(months) => months * 30,
            Self::Years(years) => years * 365,
        };
        let offset = time::Duration::days(days as i64);
        let date = time::OffsetDateTime::now_local()
            .unwrap_or(time::OffsetDateTime::now_utc())
            .date()
            .checked_sub(offset);
        match date {
            Some(date) => Date::from(date),
            None => Date::min()
        }
    }
}

impl str::FromStr for Period {
    type Err = GlobalError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (count_str, period_str) = value.split_once(" ").unwrap_or(("1", value));
        let count = count_str
            .parse()
            .map_err(|_| GlobalError::parse("period", value.to_string(), Self::FORMAT_HINT))?;
        match period_str {
            "days" | "day" => Ok(Self::Days(count)),
            "weeks" | "week" => Ok(Self::Weeks(count)),
            "months" | "month" => Ok(Self::Months(count)),
            "years" | "year" => Ok(Self::Years(count)),
            _ => Err(GlobalError::parse("period", value.to_string(), Self::FORMAT_HINT))
        }
    }
}

/// Represent a calendar date
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    const YEAR_SIZE: usize = 2;
    const MONTH_SIZE: usize = 1;
    const DAY_SIZE: usize = 1;
    pub const SIZE: usize = Self::YEAR_SIZE + Self::MONTH_SIZE + Self::DAY_SIZE;
    /// Hint on how to format a date as a string.
    const FORMAT_HINT: &'static str
        = "`dd-mm-yyyy` where `dd` are days, `mm` are months and `yyyy` are years";

    /// Minimum valid date
    pub fn min() -> Self {
        Self {
            year: 0,
            month: 0,
            day: 0,
        }
    }

    /// Maximum valid date
    pub fn max() -> Self {
        Self {
            year: u16::MAX,
            month: 12,
            day: 31
        }
    }

    /// Returns today's date
    pub fn today() -> Self {
        let today = time::OffsetDateTime::now_local()
            .unwrap_or(time::OffsetDateTime::now_utc())
            .date();
        Self::from(today)
    }

    pub fn to_le_bytes(&self) -> [u8; Self::SIZE] {
        let year_bytes = self.year.to_le_bytes();
        [year_bytes[0], year_bytes[1], self.month, self.day]
    }

}

impl From<time::Date> for Date {
    fn from(date: time::Date) -> Self {
        let (mut year, month, day) = date.to_calendar_date();
        if year < 0 || year > u16::MAX as i32 {
            year = 0;
        }
        Self {
            year: year as u16,
            month: month.into(),
            day,
        }
    }
}

impl std::convert::TryFrom<&[u8]> for Date {
    type Error = GlobalError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() != Self::SIZE {
            return Err(GlobalError::wrong_byte_len("date", buffer.len(), Self::SIZE))
        }
        let year_bytes = buffer[..2]
            .try_into()
            .expect("buffer len was checked previously");
        Ok(Self {
            year: u16::from_le_bytes(year_bytes),
            month: buffer[2],
            day: buffer[3],
        })
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl str::FromStr for Date {
    type Err = GlobalError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let error = GlobalError::parse("date", value.to_string(), Self::FORMAT_HINT);
        let (year_str, month_and_day_str) = value.split_once("-")
            .ok_or_else(|| error.clone())?;
        let (month_str, day_str) = month_and_day_str.split_once("-")
            .ok_or_else(|| error.clone())?;

        Ok(Self {
            year: year_str.parse::<u16>().map_err(|_| error.clone())?,
            month: month_str.parse::<u8>().map_err(|_| error.clone())?,
            day: day_str.parse::<u8>().map_err(|_| error.clone())?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    hours: u8,
    minutes: u8,
}

impl Time {
    const HOURS_SIZE: usize = 1;
    const MINUTES_SIZE: usize = 1;
    /// Number of bytes to represent time when encoded
    pub const SIZE: usize = Self::HOURS_SIZE + Self::MINUTES_SIZE;
    /// How a time which is not saved is represented when encoded
    pub const NONE_DATE: [u8; 2] = [0xFF, 0xFF];
    /// Hint how to format time as a string.
    pub const FORMAT_HINT: &'static str
        = "`hh:mm` where `hh` represents the hours and `mm` represents the minutes";

    pub fn now() -> Self {
        let (hours, minutes, _) = time::OffsetDateTime::now_local()
            .unwrap_or(time::OffsetDateTime::now_utc())
            .time()
            .as_hms();
        Self {
            hours,
            minutes
        }
    }

    /// Returns a time instance, which when saved to a file it represent the abscence of time.
    ///
    /// When both `hours` and `minutes` are at the maximum value (255), it means the time does not
    /// exists.
    pub fn none() -> Self {
        Self {
            hours: u8::MAX,
            minutes: u8::MAX,
        }
    }

    pub fn to_le_bytes(&self) -> [u8; Self::SIZE] {
        [self.hours, self.minutes]
    }
}

impl default::Default for Time {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:02}", self.hours, self.minutes)
    }
}

impl std::convert::TryFrom<&[u8]> for Time {
    type Error = GlobalError;
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() != Self::SIZE {
            return Err(GlobalError::wrong_byte_len("time", buffer.len(), Self::SIZE))
        }
        Ok(Self {
            hours: buffer[0],
            minutes: buffer[1],
        })
    }
}

impl str::FromStr for Time {
    type Err = error::GlobalError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (hours_str, minutes_str) = value.split_once(":")
            .ok_or_else(|| GlobalError::parse("time", value.to_string(), Time::FORMAT_HINT))?;
        let hours = hours_str.parse::<u8>()
            .map_err(|_| GlobalError::parse("time", value.to_string(), Time::FORMAT_HINT))?;
        let minutes = minutes_str.parse::<u8>()
            .map_err(|_| GlobalError::parse("time", value.to_string(), Time::FORMAT_HINT))?;
        Ok(Self {
            hours,
            minutes
        })
    }
}
