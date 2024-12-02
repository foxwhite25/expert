use log::info;
use rand::prelude::SliceRandom;
use tracing::error;
use crate::command::handle_help;
use crate::Context;

pub async fn handle_rule(seg: &[&str], ctx: &mut Context) {
    match seg { 
        ["list", ..] => {
            println!("{}", ctx.list_rules());
        }
        ["shuffle", ..] => {
            ctx.rules.shuffle(&mut rand::thread_rng());
            info!("Successfully shuffled rule");
        }
        ["add", rule, output, ..] => {
            if let Err(e) = ctx.add_rule(rule, output) {
                error!("Error while adding new rules: {}", e);
                return;
            }
            info!("Successfully added rule with condition {} and output {}", rule, output);
        }
        ["add", ..] => {
            error!("用法：add <规则> <输出>");
            error!("用例：rule add fact1|(fact2&fact3) output1,output2")
        }
        ["remove", idx, ..] => {
            if let Err(e) = ctx.remove_rule(idx) {
                error!("Error while removing new rules: {}", e);
                return;
            }
            info!("Successfully removed rule with idx {}", idx);
        }
        ["remove", ..] => {
            error!("用法：remove <规则ID>");
        }
        [] => {
            handle_help(&["rule"]).await;
        }
        [x, ..] => {
            error!("未知子命令: {}", x)
        } 
    }
}