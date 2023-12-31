///! Track your time working on projects directly from the CLI. Categorize and add a description to
///! what you did and later export your timesheet to different formats.

mod config;
mod time;
mod error;
mod tag;
mod pnch;

use clap::{Parser, Subcommand, Args};
use error::GlobalError;
use std::{fs, io::Read};

const APP_NAME: &'static str = "pnch";

pub mod storage {
    use super::*;

    /// Get a file path for a file that is in the app storage.
    pub fn build_path(file: &str) -> Result<String, GlobalError> {
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

    /// Load the content from a file a returns it.
    pub fn load(file: &str) -> Result<Vec<u8>, GlobalError> {
        let path = build_path(file)?;
        let mut buffer = Vec::new();
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(path)
            .map_err(|_| GlobalError::fs("load", file))?
            .read_to_end(&mut buffer)
            .map_err(|_| GlobalError::fs("load", file))?;
        Ok(buffer)
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
    /// Punch in. This starts a new entry and record the time that the command was called. A tag
    /// and a description can be added while pnching in. It can also be added when pnching out
    /// later on. To add a tag and description, one most specify it with the following format:
    /// "my_tag/The description of my task". Everything before the forward slash is a tag and
    /// everything afterwards is the description. For more information, use `pnch in --help`.
    #[command(verbatim_doc_comment)]
    In(Entry),

    /// Punch out. This closes an entry that was previously opened with `pnch in`. If it was not
    /// specified while pnching in, a tag and a description can be added. To add a tag and
    /// description, one most specify it with the following format: "my_tag/The description of my
    /// task". Everything before the forward slash is a tag and everything afterwards is the
    /// description. For more information, use `pnch out --help`.
    #[command(verbatim_doc_comment)]
    Out(Entry),

    /// Edit or add the tag and description for a currently opened pnch. For more information, use
    /// One most specify it with the following format: "my_tag/The description of my task".
    /// Everything before the forward slash is a tag and everything afterwards is the description.
    /// For more information, use `pnch edit --help`.
    #[command(verbatim_doc_comment)]
    Edit {
        /// The description is used to describe the entry. While specifying the description, it is also
        /// possible to specify a tag. For example, a description could be specify as
        /// "ISSUE-123/The issue is fixed". The tag is the value specified before the forward slash
        /// (`/`) and the description is everything after. In the example above, "ISSUE-123" would be
        /// the tag and "The issue was fixed" would be the description of the issue.
        #[arg(verbatim_doc_comment)]
        description: Option<pnch::Description>,

        /// Specify the id for the entry to edit. The id can be found when listing entries with
        /// `pnch ls`
        #[arg(long)]
        id: Option<u32>,

        /// Specify the new start time of the entry to edit. The format should be `hh:mm` where
        /// `hh` represent hours and `mm` represent minutes. The default value is the current local
        /// time.
        #[arg(long)]
        r#in: Option<time::Time>,

        /// Specify the new start time of the entry to edit. The format should be `hh:mm` where
        /// `hh` represent hours and `mm` represent minutes. The default value is the current local
        /// time. This option is only valid when `--id` is specified (When it is not specified,
        /// simply use `pnch out --time ...`).
        #[arg(long)]
        out: Option<time::Time>,
    },

    /// List and print pnch entries. A filter can be added to only show a subset of pnchs. For
    /// example to show pnchs from the last two weeks, the command used would be
    /// `pnch ls --last 2 weeks`. If multiple period filters (`--from`, `--to`, `--since` and
    /// `--last`) are used, they act as unions and all pnchs that match at least one filter will be
    /// returned. The `--tag` filter will take this list and only returns pnchs that match the
    /// specified tag. It is also possible to format the output as a pretty print or in a csv
    /// format. By default, only the entries from the last 14 days are listed. To change this
    /// value, use `pnch config ls-default-period "28 days"`. For more information, use `pnch ls
    /// --help`.
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
    Config {
        key: String,
        value: String,
    }
}

#[derive(Args, Debug)]
pub struct Entry {
    /// The description is used to describe the entry. While specifying the description, it is also
    /// possible to specify a tag. For example, a description could be specify as
    /// "ISSUE-123/The issue is fixed". The tag is the value specified before the forward slash
    /// (`/`) and the description is everything after. In the example above, "ISSUE-123" would be
    /// the tag and "The issue was fixed" would be the description of the issue.
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
    let mut config = config::Config::load()?;

    match args.command {
        Commands::In(Entry { description, time }) => {
            let (tag, description) = description
                .map(|d| (d.tag.map(|t| tags.get_or_insert(t)), Some(d.description)))
                .unwrap_or_else(|| (None, None));
            let id = pnchs.0.len();
            pnchs._in(pnch::Pnch::new(id as u32, time, tag, description))?;
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
        Commands::Edit { description, id, r#in, out } => {
            let pnch = match id {
                Some(id) => pnchs.get(id),
                _ => pnchs.get_last(),
            };
            match pnch {
                Some(pnch) => {
                    if let Some(out) = out {
                        pnch.out = Some(out);
                    }
                    if let Some(_in) = r#in {
                        pnch._in = _in;
                    }
                    if let Some(description) = description {
                        let tag = description.tag.map(|t| tags.get_or_insert(t));
                        pnch.tag = tag;
                        pnch.description = Some(description.description);
                    }
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
                .unwrap_or(config.ls_default_period)
                .to_date_since_today();
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
                Some(pnch::Format::List) => println!("{pnchs}"),
                _ => println!("{}", pnchs.into_table())
            }
        }
        Commands::Config { key, value } => {
            config.try_set(&key, &value)?;
            config.save()?;
            println!("The config was updated.");
        }
    }
    Ok(())
}
