use colored::Colorize;
use crate::command::db::handle_db;
use crate::Context;
use tracing::error;
use crate::command::rule::handle_rule;
use crate::rule::Facts;

mod db;
mod rule;

pub fn print_header() {
    println!(
        "专家系统 v{} by {}",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );
    println!("使用 {} {} 可以查询更多信息", "help".cyan(), "<命令>".yellow());
    println!();
}

async fn handle_help(seg: &[&str]) {
    print_header();
    match seg {
        ["help", ..] => {
            println!("输出帮助信息");
            println!("用法: help <命令>");
        }
        ["quit", ..] => {
            println!("退出程序");
            println!("用法: quit");
        }
        ["rules", ..] => {
            println!("查看或修改规则库中的规则");
            println!("用法: rule <子命令>");
            println!("子命令:");
            println!("  list: 列出所有规则");
            println!("  add <规则> <输出>: 添加新规则");
            println!("  remove <规则ID>: 删除指定规则");
        }
        ["test", ..] => {
            println!("输入一系列的事实进行推论");
            println!("用法: test <事实>");
            println!("示例: test fact1 fact2");
        }
        ["db", ..] => {
            println!("查看sqlite数据库信息");
            println!("用法: db <子命令>");
            println!("子命令:");
            println!("  connect <路径>: 连接数据库");
            println!("  close: 断开数据库连接");
            println!("  status: 查看数据库状态");
            println!("  load: 从数据库加载规则库");
            println!("  sync: 保存规则库到数据库");
            println!("  reset: 重置数据库");
        }
        [] => {
            println!("命令:");
            println!("  help: 输出此帮助信息");
            println!("  quit: 退出程序");
            println!("  rule: 查看或修改规则库中的规则");
            println!("  test: 输入一系列的事实进行推论");
            println!("  db: 查看数据库信息");
        }
        _ => {
            error!("未知命令: {}", seg[0])
        }
    }
}

pub async fn handle_command(line: String, ctx: &mut Context) -> anyhow::Result<bool> {
    let segments: Vec<&str> = line.split(" ").collect();
    match segments.as_slice() {
        ["help", ..] => {
            handle_help(&segments[1..]).await;
        }
        ["quit", ..] => {
            return Ok(true);
        }
        ["rule", ..] => {
            handle_rule(&segments[1..], ctx).await;
        }
        ["test", ..] => {
            let mut facts = Facts::new(&segments[1..]);
            facts.deduce(&ctx.rules);
        }
        ["db", ..] => {
            handle_db(&segments[1..], ctx).await;
        }
        _ => {}
    }
    Ok(false)
}
