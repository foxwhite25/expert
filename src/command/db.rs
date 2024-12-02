use crate::command::handle_help;
use crate::Context;
use tabled::settings::object::Rows;
use tabled::settings::{Alignment, Style};
use tracing::{error, info};

pub(crate) async fn handle_db(seg: &[&str], ctx: &mut Context) {
    match seg {
        ["connect", ..] => {
            if let Some(path) = seg.get(1) {
                connect(path, ctx).await;
            } else {
                error!("用法：connect <路径>")
            }
        }
        ["close", ..] => {
            close(ctx).await;
        }
        ["status", ..] => {
            status(ctx).await;
        }
        ["reset", ..] => {
            reset(ctx).await;
        }
        ["load", ..] => {
            load(ctx).await;
        }
        ["sync", ..] => {
            sync(ctx).await;
        }
        [] => {
            handle_help(&["db"]).await;
        }
        [x, ..] => {
            error!("未知子命令: {}", x)
        }
    }
}
async fn connect(path: &str, ctx: &mut Context) {
    if ctx.db.is_some() {
        error!("db connection is already established, close it with db close");
        return;
    }
    info!("Initializing database {}", path);

    let result = ctx.connect(path).await;
    if let Err(e) = result {
        error!("Error while creating connection: {}", e)
    } else {
        info!("Successfully established connection to {}", path);
    }
}

async fn close(ctx: &mut Context) {
    if ctx.db.is_none() {
        error!("No established db connection, use db connect first");
        return;
    }
    ctx.db = None;
    info!("Successfully closed connection");
}

async fn status(ctx: &mut Context) {
    let Some(db) = ctx.db.as_ref() else {
        error!("No established db connection, use db connect first");
        return;
    };
    let rules = db.load_rules_raw().await;
    let rules = match rules {
        Ok(r) => r,
        Err(e) => {
            error!("Error while reading db: {}", e);
            return;
        }
    };
    let mut builder = tabled::builder::Builder::default();
    builder.push_record(["id", "condition", "output"]);
    for rule in rules {
        builder.push_record([rule.0.to_string(), rule.1, rule.2]);
    }
    let table = builder
        .build()
        .with(Style::rounded())
        .modify(Rows::new(1..), Alignment::left())
        .to_string();
    println!("{}", table);
}

async fn reset(ctx: &mut Context) {
    let Some(db) = ctx.db.as_ref() else {
        error!("No established db connection, use db connect first");
        return;
    };
    info!("Resetting database");
    if let Err(e) = db.reset().await {
        error!("Error while resetting db: {}", e);
    } else {
        info!("Database reset complete");
    };
}

async fn load(ctx: &mut Context) {
    let Some(db) = ctx.db.as_ref() else {
        error!("No established db connection, use db connect first");
        return;
    };
    info!("Loading database");
    let rules = db.load_rules().await;
    let rules = match rules {
        Ok(rules) => {rules}
        Err(e) => {
            error!("Error while reading db: {}", e);
            return;
        }
    };
    ctx.rules = rules;
    info!("Successfully loaded {} rules", ctx.rules.len());
}

async fn sync(ctx: &mut Context) {
    let Some(db) = ctx.db.as_ref() else {
        error!("No established db connection, use db connect first");
        return;
    };
    info!("Syncing database");
    if let Err(e) = db.reset().await {
        error!("Error while resetting db: {}", e);
        return;
    }
    if let Err(e) = db.save_rules(&ctx.rules).await {
        error!("Error while saving rules: {}", e);
        return;
    }
    info!("Database sync complete");
}