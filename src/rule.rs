use anyhow::anyhow;
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;
use log::info;

pub struct Facts(Vec<String>);

impl Facts {
    pub fn new(vec: &[&str]) -> Facts {
        Facts(vec.iter().map(|s| s.to_string()).collect())
    }

    fn recall(&self, fact: &str) -> bool {
        self.0.iter().any(|x| x == fact)
    }

    fn test_if(&self, condition: &Condition) -> bool {
        condition.matches(&self.0)
    }

    fn remember(&mut self, fact: &str) -> bool {
        if self.recall(fact) {
            return false;
        }
        self.0.push(fact.to_string());
        true
    }

    pub fn step_forward(&mut self, rules: &[Rule]) -> bool {
        let mut any_rule_matched = false;

        for rule in rules {
            if self.test_if(&rule.condition) {
                let matched = rule.output.iter().any(|fact| self.remember(fact));
                any_rule_matched |= matched;
                if matched {
                    info!("Because {} is valid, add outputs: {:?}", rule.condition.to_string(), rule.output);
                }
                
            }
        }

        any_rule_matched
    }

    pub fn deduce(&mut self, rules: &[Rule]) -> usize {
        let mut step = 0;
        info!("Initial facts: {:?}", self.0);
        while self.step_forward(rules) {
            step += 1;
            info!("Cycle {}, facts: {:?}", step, self.0);
        }
        
        info!("Deduction complete, used {} cycle, facts: {:?}", step, self.0);
        step
    }
}

impl From<Vec<&str>> for Facts {
    fn from(vec: Vec<&str>) -> Facts {
        Facts(vec.iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<String>> for Facts {
    fn from(vec: Vec<String>) -> Facts {
        Facts(vec)
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub(crate) condition: Condition,
    pub(crate) output: Vec<String>,
}

impl TryFrom<(i64, String, String)> for Rule {
    type Error = anyhow::Error;

    fn try_from(value: (i64, String, String)) -> Result<Self, Self::Error> {
        let (_, condition, output) = value;
        let condition = Condition::from_str(&condition).map_err(|s| anyhow!(s))?;
        let output = output.split(",").map(|s| s.to_string()).collect();
        Ok(Rule { condition, output })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
    Fact(String),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

impl Condition {
    pub fn fact(s: &str) -> Condition {
        Condition::Fact(s.to_string())
    }

    pub fn and(self, rhs: Condition) -> Condition {
        Condition::And(Box::new(self), Box::new(rhs))
    }

    pub fn or(self, rhs: Condition) -> Condition {
        Condition::Or(Box::new(self), Box::new(rhs))
    }

    pub fn not(self) -> Condition {
        Condition::Not(Box::new(self))
    }

    pub fn matches(&self, facts: &Vec<String>) -> bool {
        match self {
            Condition::Fact(obj) => facts.contains(obj),
            Condition::And(lhs, rhs) => lhs.matches(facts) && rhs.matches(facts),
            Condition::Or(lhs, rhs) => lhs.matches(facts) || rhs.matches(facts),
            Condition::Not(inner) => !inner.matches(facts),
        }
    }
}

impl FromStr for Condition {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars().peekable();
        parse_or(&mut chars).map_err(|e| anyhow!(e))
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Fact(fact) => write!(f, "{}", fact),
            Condition::And(lhs, rhs) => write!(f, "({} & {})", lhs, rhs),
            Condition::Or(lhs, rhs) => write!(f, "({} | {})", lhs, rhs),
            Condition::Not(inner) => write!(f, "!{}", inner),
        }
    }
}

fn parse_or(chars: &mut Peekable<Chars>) -> Result<Condition, String> {
    let mut lhs = parse_and(chars)?;

    skip_whitespace(chars);
    while let Some(&c) = chars.peek() {
        if c == '|' {
            chars.next(); // consume '|'
            let rhs = parse_and(chars)?;
            lhs = lhs.or(rhs);
            skip_whitespace(chars);
        } else {
            break;
        }
    }

    Ok(lhs)
}

fn parse_and(chars: &mut Peekable<Chars>) -> Result<Condition, String> {
    let mut lhs = parse_not(chars)?;

    skip_whitespace(chars);
    while let Some(&c) = chars.peek() {
        if c == '&' {
            chars.next(); // consume '&'
            let rhs = parse_not(chars)?;
            lhs = lhs.and(rhs);
            skip_whitespace(chars);
        } else {
            break;
        }
    }

    Ok(lhs)
}

fn parse_not(chars: &mut Peekable<Chars>) -> Result<Condition, String> {
    skip_whitespace(chars);
    if let Some(&c) = chars.peek() {
        if c == '!' {
            chars.next(); // consume '!'
            let rule = parse_not(chars)?;
            return Ok(Condition::not(rule));
        }
    }
    parse_primary(chars)
}

fn parse_primary(chars: &mut Peekable<Chars>) -> Result<Condition, String> {
    skip_whitespace(chars);

    if let Some(&c) = chars.peek() {
        match c {
            '(' => {
                chars.next(); // consume '('
                let rule = parse_or(chars)?;
                skip_whitespace(chars);
                if chars.next() != Some(')') {
                    return Err("Expected ')'".into());
                }
                Ok(rule)
            }
            _ => parse_fact(chars),
        }
    } else {
        Err("Unexpected end of input".into())
    }
}

fn parse_fact(chars: &mut Peekable<Chars>) -> Result<Condition, String> {
    let mut obj = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            obj.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if obj.is_empty() {
        Err("Expected fact".into())
    } else {
        Ok(Condition::fact(&obj))
    }
}

fn skip_whitespace(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fact() {
        let rule = "fact".parse::<Condition>().unwrap();
        assert_eq!(rule, Condition::fact("fact"));
    }

    #[test]
    fn test_parse_not() {
        let rule = "!fact".parse::<Condition>().unwrap();
        assert_eq!(rule, Condition::fact("fact").not());
    }

    #[test]
    fn test_parse_and() {
        let rule = "fact1 & fact2".parse::<Condition>().unwrap();
        assert_eq!(rule, Condition::fact("fact1").and(Condition::fact("fact2")));
    }

    #[test]
    fn test_parse_or() {
        let rule = "fact1 | fact2".parse::<Condition>().unwrap();
        assert_eq!(rule, Condition::fact("fact1").or(Condition::fact("fact2")));
    }

    #[test]
    fn test_parse_complex_expression() {
        let rule = "!(fact1 & fact2) | fact3".parse::<Condition>().unwrap();
        assert_eq!(
            rule,
            Condition::fact("fact1")
                .and(Condition::fact("fact2"))
                .not()
                .or(Condition::fact("fact3"))
        );
    }

    #[test]
    fn test_parse_with_parentheses() {
        let rule = "(fact1 | fact2) & fact3".parse::<Condition>().unwrap();
        assert_eq!(
            rule,
            Condition::fact("fact1")
                .or(Condition::fact("fact2"))
                .and(Condition::fact("fact3"))
        );
    }

    #[test]
    fn test_invalid_input() {
        assert!("& fact".parse::<Condition>().is_err());
        assert!("fact1 &".parse::<Condition>().is_err());
        assert!("(fact1 & fact2".parse::<Condition>().is_err());
    }

    #[test]
    fn test_match_fact() {
        let rule = Condition::fact("fact1");
        let facts = vec!["fact1".to_string(), "fact2".to_string()];
        assert!(rule.matches(&facts));
    }

    #[test]
    fn test_match_not_fact() {
        let rule = Condition::fact("fact1").not();
        let facts = vec!["fact2".to_string(), "fact3".to_string()];
        assert!(rule.matches(&facts));
    }

    #[test]
    fn test_match_and() {
        let rule = Condition::fact("fact1").and(Condition::fact("fact2"));
        let facts = vec!["fact1".to_string(), "fact2".to_string()];
        assert!(rule.matches(&facts));
    }

    #[test]
    fn test_match_or() {
        let rule = Condition::fact("fact1").or(Condition::fact("fact3"));
        let facts = vec!["fact2".to_string(), "fact3".to_string()];
        assert!(rule.matches(&facts));
    }

    #[test]
    fn test_no_match() {
        let rule = Condition::fact("fact4");
        let facts = vec!["fact1".to_string(), "fact2".to_string()];
        assert!(!rule.matches(&facts));
    }

    #[test]
    fn test_step_forward() {
        let mut facts = Facts::new(&[]);
        facts.remember("fact1");

        let rules = vec![
            Rule {
                condition: Condition::fact("fact1"),
                output: vec!["fact2".to_string()],
            },
            Rule {
                condition: Condition::fact("fact2"),
                output: vec!["fact3".to_string()],
            },
            Rule {
                condition: Condition::fact("fact4"),
                output: vec!["fact5".to_string()],
            },
        ];

        let result = facts.step_forward(&rules);
        assert!(result);
        assert!(facts.recall("fact2"));
        assert!(facts.recall("fact3"));
        assert!(!facts.recall("fact5"));
    }

    #[test]
    fn test_no_matching_rules() {
        let mut facts = Facts::new(&[]);
        facts.remember("fact1");

        let rules = vec![Rule {
            condition: Condition::fact("fact4"),
            output: vec!["fact5".to_string()],
        }];

        let result = facts.step_forward(&rules);
        assert!(!result);
        assert!(!facts.recall("fact5"));
    }

    #[test]
    fn test_deduce() {
        let mut facts = Facts::new(&[]);
        facts.remember("fact1");

        let rules = vec![
            Rule {
                condition: Condition::fact("fact2"),
                output: vec!["fact3".to_string()],
            },
            Rule {
                condition: Condition::fact("fact1"),
                output: vec!["fact2".to_string()],
            },
            Rule {
                condition: Condition::fact("fact3"),
                output: vec!["fact4".to_string()],
            },
        ];

        let step = facts.deduce(&rules);

        assert!(facts.recall("fact2"));
        assert!(facts.recall("fact3"));
        assert!(facts.recall("fact4"));
        assert_eq!(step, 2)
    }

    #[test]
    fn test_deduce_no_changes() {
        let mut facts = Facts::new(&[]);
        facts.remember("fact1");

        let rules = vec![Rule {
            condition: Condition::fact("fact5"),
            output: vec!["fact6".to_string()],
        }];

        facts.deduce(&rules);

        assert!(!facts.recall("fact6"));
    }

    #[test]
    fn test_condition_to_string() {
        let condition = Condition::fact("fact1")
            .and(Condition::fact("fact2").or(Condition::not(Condition::fact("fact3"))));

        assert_eq!(condition.to_string(), "(fact1 & (fact2 | !fact3))");
        let parsed = condition.to_string().parse::<Condition>().unwrap();
        assert_eq!(parsed, condition);
    }
}
