use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "1.0", author = "quantumish")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(long, default_value="localhost")]
    pub ip: String,
    #[clap(subcommand)]
    pub subcmd: QueryCmd,
}

#[derive(Clap)]
pub enum QueryCmd {
    Add(Add),
    Get(Get),
    Log(Log),
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Add {
    #[clap(subcommand)]
    pub subcmd: EntityCmd,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum EntityCmd {
    Block(Block),
    Tag(Tag),
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Block {
    #[clap(short)]
    pub name: String,
    #[clap(short)]
    pub tags: String,
    #[clap(short)]
    pub project: String,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Tag {
    #[clap(short)]
    pub name: String,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Get {
    #[clap(short)]
    pub rel: Option<usize>,
    #[clap(short)]
    pub id: Option<String>,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Log {
    // #[clap(long)]
    // compact: bool,
}
