use crate::time;
use std::{fmt, error};


#[derive(Debug, Clone)]
pub struct GlobalError {
    error: String,
    hint: Option<String>,
}

impl GlobalError {
    pub fn parse(typ: &'static str, found: String, format_hint: &'static str) -> Self {
        let mut error = format!("The {typ} was specified with the wrong format. ");
        error.push_str(&format!("The given value was `{found}`"));
        Self {
            error,
            hint: Some(format!("The format should be {format_hint}"))
        }
    }

    pub fn wrong_byte_len(typ: &'static str, actual: usize, expected: usize) -> Self {
        let mut error = format!("Could not decode the {typ}. ");
        error.push_str(&format!("Expected {expected} bytes, but got {actual} bytes."));
        Self {
            error,
            hint: None,
        }
    }

    pub fn desc_only_tag(tag: String) -> Self {
        let mut error = format!("You must specify a description with your tag.\n");
        error.push_str(&format!("    tag: {tag}\n    description: not specified"));
        let hint = String::from("To specify a description, add content after the first forward slash in `pnch in \"my tag/my desription\"`");
        Self {
            error,
            hint: Some(hint)
        }
    }

    pub fn fs(action: &str, typ: &str) -> Self {
        Self {
            error: format!("Could not {action} the {typ} database."),
            hint: Some(String::from("This is probably a bug, you should report it to the bug tracker."))
        }
    }

    pub fn desc_already_specified(tag: &str, description: &str) -> Self {
        let mut error = String::from("A tag and message are already link to the entry.\n");
        error.push_str(&format!("    tag: {tag}"));
        error.push_str(&format!("    description: {description}"));
        Self {
            error,
            hint: Some(String::from("To edit the current entry, use `pnch edit tag/message`"))
        }
    }

    pub fn desc_not_specified() -> Self {
        let mut hint = String::from("To add a tag or message to the current entry, either use");
        hint.push_str(" `pnch add tag/message` or `pnch out tag/message`.\n");
        hint.push_str("You can also add a description while pnching in with `pnch in \"tag/description\"`.");
        Self {
            error: String::from("No message or tag were specified."),
            hint: Some(hint)
        }
    }

    pub fn pnch_already_closed() -> Self {
        Self {
            error: String::from("The entry is already closed."),
            hint: Some(String::from("To update the out time of an entry, use `pnch edit --time ...`")),
        }
    }

    pub fn pnch_not_exists() -> Self {
        Self {
            error: String::from("No pnch exists."),
            hint: Some(String::from("To open a new pnch, use `pnch in \"my tag/my description\"`")),
        }
    }

    pub fn pnch_not_open() -> Self {
        Self {
            error: String::from("No pnch seems to be opened."),
            hint: Some(String::from("To open a new pnch, use `pnch in \"my tag/my description\"`")),
        }
    }

    pub fn formatting(typ: &str) -> Self {
        Self {
            error: format!("Could not format data with the formatting option `{typ}`"),
            hint: Some(String::from("This is probably a bug, you should report it to the bug tracker."))
        }
    }

    pub fn ls_uncomplete_range() -> Self {
        Self {
            error: format!("The specified range was not complete."),
            hint: Some(String::from("When defining a range both the `--from DATE` and `--to DATE` should be specified.")),
        }
    }

    pub fn pnch_out_before_in(_in: time::Time, out: time::Time) -> Self {
        Self {
            error: format!("The `out` time cannot be before the `in` time. (in: {_in}, out: {out})"),
            hint: None
        }
    }

    pub fn pnch_already_open() -> Self {
        Self {
            error: String::from("A pnch is already open."),
            hint: Some(String::from("Before pnching in, close the pnch with `pnch out`"))
        }
    }
}

impl From<std::string::FromUtf8Error> for GlobalError {
    fn from(_error: std::string::FromUtf8Error) -> Self {
        Self {
            error: format!("Could not decode a string from the database."),
            hint: Some(String::from("This is probably a bug, you should report it to the bug tracker."))
        }
    }
}

impl fmt::Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[ERROR]")?;
        writeln!(f, "  {}", self.error)?;
        if let Some(hint) = &self.hint {
            writeln!(f, "[HINT]")?;
            writeln!(f, "  {hint}")?;
        }
        Ok(())
    }
}

impl error::Error for GlobalError {}
