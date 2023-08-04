mod time;
mod error;
mod tag;
mod pnch;

use clap::{Parser, Subcommand, Args};
use error::GlobalError;
use std::fs;

const APP_NAME: &'static str = "pnch";

/// Get a file path for a file that is in the app storage.
pub fn get_file_path(file: &str) -> Result<String, GlobalError> {
    let mut path = directories::BaseDirs::new()
        .map_or(Err(GlobalError::fs("load", file)), |base_dirs| {
            let mut path = base_dirs.data_dir().to_owned();
            path.push(APP_NAME);
            fs::create_dir_all(path.clone())
                .map_err(|_| GlobalError::fs("create dir", file))?;
            Ok(path)
        })?;
    path.push(file);
    match path.to_str() {
        Some(path) => Ok(path.to_string()),
        _ => return Err(GlobalError::fs("load", file))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = None, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Punch in. For more information, use `pnch in --help`.
    #[command(verbatim_doc_comment)]
    In(Entry),
    /// Punch out. For more information, use `pnch out --help`.
    #[command(verbatim_doc_comment)]
    Out(Entry),
    /// Edit or add the tag and description for a currently opened pnch. For more information, use
    /// `pnch edit --help`.
    Edit {
        #[arg(verbatim_doc_comment)]
        description: pnch::Description,
    },
    /// List and print pnch entries. A filter can be added to only show a subset of pnchs. For
    /// example to show pnchs from the last two weeks, the command used would be 
    /// `pnch ls --last 2 weeks`. If multiple period filters (`--from`, `--to`, `--since` and
    /// `--last`) are used, they act as unions and all pnchs that match at least one filter will be
    /// returned. The `--tag` filter will take this list and only returns pnchs that match the
    /// specified tag. It is also possible to format the output as a pretty print or in a csv
    /// format. For more information, use `pnch ls --help`.
    #[command(verbatim_doc_comment)]
    Ls {
        /// Get all pnchs since the specified date in the yyyy-mm-dd format
        #[arg(long, short)]
        since: Option<time::Date>,
        /// Get all pnchs for the last n period. A period can be `days`, `weeks`, `months` or
        /// `years`.
        #[arg(long, short)]
        last: Option<time::Period>,
        /// Specify a range of dates in combination with the `to` flag. Date is specified with the
        /// yyyy-mm-dd format.
        #[arg(long, short)]
        from: Option<time::Date>,
        /// Specify a range of dates in combination with the `from` flag. Date is specified with the
        /// yyyy-mm-dd format.
        #[arg(long, short)]
        to: Option<time::Date>,
        /// Filter only entries from a specific tag
        #[arg(long)]
        tag: Option<String>,
        /// Specify how to format the output. The value should be one of `pretty` or `csv`. The
        /// default is `pretty`.
        #[arg(long)]
        format: Option<pnch::Format>
    },
}

#[derive(Args, Debug)]
pub struct Entry {
    /// Direclty specify the tag and description of an entry without using flags. The tag and 
    /// the description are separated with a forward slash and the tag is appearing first. When
    /// the flags are specified, they overwrite this field extracted value.
    /// `pnch in "my tag/my message"` is the same as `pnch in --tag "my tag" --description "my
    /// description"`.
    /// To only specify the tag use this format: `my tag`. To only specify the description use
    /// this format: `/my description`.
    #[arg(verbatim_doc_comment)]
    description: Option<pnch::Description>,
    /// Manually specify time. The format should be `hh:mm` where `hh` represent hours and
    /// `mm` represent minutes. The default value is the current local time.
    #[arg(long, verbatim_doc_comment, default_value_t)]
    time: time::Time,
}

fn main() {
    let args = Cli::parse();
    if let Err(err) = run(args) {
        eprintln!("{err}");
    }
}

fn run(args: Cli) -> Result<(), GlobalError> {
    let mut tags = tag::Tags::load()?;
    let mut pnchs = pnch::Pnchs::load(&tags)?;

    match args.command {
        Commands::In(Entry { description, time }) => {
            let (tag, description) = description
                .map(|d| (d.tag.map(|t| tags.get_or_insert(t)), Some(d.description)))
                .unwrap_or_else(|| (None, None));
            pnchs._in(pnch::Pnch::new(time, tag, description))?;
            pnchs.save()?;
            tags.save()?;
            println!("You are now pnched in.");
        }
        Commands::Out(Entry { description, time }) => {
            match pnchs.get_last() {
                Some(pnch) => {
                    let (tag, description) = description
                        .map(|d| (d.tag.map(|t| tags.get_or_insert(t)), Some(d.description)))
                        .unwrap_or_else(|| (None, None));
                    pnch.out(time, tag, description)?;
                    pnchs.save()?;
                    tags.save()?;
                    println!("You are now pnched out.");
                }
                _ => {
                    return Err(GlobalError::pnch_not_open());
                }
            }
        }
        Commands::Edit { description } => {
            match pnchs.get_last() {
                Some(pnch) if pnch.out.is_some() => {
                    return Err(GlobalError::pnch_already_closed());
                }
                Some(pnch) => {
                    let tag = description.tag.map(|t| tags.get_or_insert(t));
                    pnch.tag = tag;
                    pnch.description = Some(description.description);
                    pnchs.save()?;
                    tags.save()?;
                    println!("The pnch was edited.");
                }
                _ => {
                    return Err(GlobalError::pnch_not_open());
                }
            }

        }
        Commands::Ls { since, last, from, to, tag, format } => {
            if from.is_some() && to.is_none() || from.is_none() && to.is_some() {
                return Err(GlobalError::ls_uncomplete_range())
            }
            let since = since.unwrap_or(time::Date::min());
            let last_as_since = last
                .map(|l| l.to_date_since_today())
                .unwrap_or(time::Date::min());
            let from = from.unwrap_or(time::Date::min());
            let to = to.unwrap_or(time::Date::max());
            let pnchs = pnch::Pnchs(pnchs
                .0
                .into_iter()
                .filter(|pnch| {
                    if pnch.date < since {
                        return false;
                    }
                    if pnch.date < last_as_since {
                        return false;
                    }
                    if pnch.date < from || pnch.date > to {
                        return false;
                    }
                    return true;
                })
                .filter(|pnch| match (&pnch.tag, &tag) {
                    (_, None) => true,
                    (Some(pnch_tag), Some(filter_tag)) => {
                        &pnch_tag.tag == filter_tag
                    }
                    _ => false

                })
                .collect::<Vec<_>>());

            match format {
                Some(pnch::Format::Csv) => println!("{}", pnchs.into_csv()?),
                _ => println!("{pnchs}")
            }
        }
    }
    Ok(())
}
