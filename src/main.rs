use crate::command::{handle_command, print_header};
use crate::db::Database;
use crate::rule::{Condition, Rule};
use anyhow::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tabled::settings::object::Rows;
use tabled::settings::{Alignment, Style};
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod command;
mod db;
mod rule;

#[derive(Debug, Default)]
pub struct Context {
    db: Option<Database>,
    rules: Vec<Rule>,
}

impl Context {
    pub fn new() -> Self {
        Context::default()
    }

    pub async fn connect(&mut self, url: &str) -> Result<()> {
        let db = Database::new(url).await?;
        self.db = Some(db);
        Ok(())
    }

    pub fn add_rule(&mut self, condition: &str, output: &str) -> Result<()> {
        let condition = condition.parse::<Condition>()?;
        let output = output.split(",").map(|x| x.to_string()).collect::<Vec<_>>();
        let rule = Rule{condition, output};
        self.rules.push(rule);
        Ok(())
    }

    pub fn list_rules(&self) -> String {
        let mut builder = tabled::builder::Builder::default();
        builder.push_record(["id", "condition", "output"]);
        for (i, rule) in self.rules.iter().enumerate() {
            builder.push_record([i.to_string(), rule.condition.to_string(), rule.output.join(",")]);
        }
        builder
            .build()
            .with(Style::rounded())
            .modify(Rows::new(1..), Alignment::left())
            .to_string()
    }

    pub fn remove_rule(&mut self, idx: &str) -> Result<()> {
        let idx = idx.parse::<usize>()?;
        self.rules.remove(idx);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    match enable_ansi_support::enable_ansi_support() {
        Ok(()) => {
            println!("\x1b[31mHello, world\x1b[0m");
        }
        Err(e) => {
            panic!("Could not enable ansi support: {}", e);
        }
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    // Start logging to console
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        info!("No previous history.");
    }
    let mut ctx = Context::new();
    print_header();

    loop {
        let readline = rl.readline(&">> ".cyan());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let quit = handle_command(line, &mut ctx).await?;
                if quit {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                info!("Exiting due to CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                info!("Exiting due to CTRL-D");
                break;
            }
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        }
        println!()
    }
    rl.save_history("history.txt")?;
    Ok(())
}
