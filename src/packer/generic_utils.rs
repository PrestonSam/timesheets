use std::{fmt::{Debug, Display}, iter::once};
use pest::{iterators::Pair, RuleType, Span};

#[derive(Clone)]
pub struct Providence<'a> {
    pub span: Span<'a>,
    pub src: &'a str,
}

impl<'a> Providence<'a> {
    pub fn as_string(&self) -> String {
        self.src.to_string()
    }
    
     pub fn as_trimmed_string(&self) -> String {
        self.src.trim().to_string()
     }
}

fn trunc(str: &str, len: usize) -> String {
    if str.len() <= len {
        format!("{:?}", str)
    } else {
        format!("{:?}..", format!("{:.*}", len, str))
    }
}

impl<'a> std::fmt::Debug for Providence<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, column) = self.span.start_pos().line_col();

        write!(f, "At {}:{}, source code: {}", line, column, trunc(self.src, 40))
    }
}


// TODO might want to simplify this as providence & rules variants have been made redundant
#[derive(Debug)]
enum PackingErrorContext<Rule> {
    Rule(Rule),
}

impl<Rule> Display for PackingErrorContext<Rule> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TODO impl<Rule> Display for PackingErrorContext<Rule> line 46 of generic_utils")
    }
}

#[derive(Debug) ]
pub struct PackingError<Variant, Rule>
where
    Variant : Debug + Display,
    Rule: Debug
{
    error: Variant,
    context: Vec<PackingErrorContext<Rule>>,
}

impl<Variant, Rule> PackingError<Variant, Rule>
where
    Variant : Debug + Display,
    Rule: Debug
{
    pub fn new(error: Variant) -> Self {
        PackingError {
            error,
            context: vec![],
        }
    }

    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.context.push(PackingErrorContext::Rule(rule));
        self
    }
}

impl<Variant, Rule> Display for PackingError<Variant, Rule>
where
    Variant : Debug + Display,
    Rule: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PACKING ERROR\n")?;
        f.write_fmt(format_args!("{}\n", self.error))?;

        self.context
            .iter()
            .map(|context| {
                f.write_fmt(format_args!("{}\n\n", context))?;

                Ok(())
            })
            .collect()
    }
}

pub trait DropRules {
    type Rule: RuleType + DropRules + Debug + Copy + Ord;

    fn get_drop_rules(&self) -> Vec<Self::Rule>;
}


#[derive(Clone)]
pub struct SyntaxTree<'a, R: RuleType> {
    pub rule: R,
    pub providence: Providence<'a>,
    pub children: Option<SyntaxChildren<'a, R>>,
}

impl<'a, R: RuleType> SyntaxTree<'a, R> {
    pub fn as_string(&self) -> String {
        self.providence.as_string()
    }
}

#[derive(Clone)]
pub enum SyntaxChildren<'a, R: RuleType> {
    One(Box<SyntaxTree<'a, R>>),
    Many(Vec<SyntaxTree<'a, R>>),
}

impl<'a, R: RuleType> SyntaxChildren<'a, R> {
    pub fn get_values(self) -> Vec<SyntaxTree<'a, R>> {
        match self {
            SyntaxChildren::One(child) => vec![*child],
            SyntaxChildren::Many(children) => children,
        }
    }
}

impl<'a, R: RuleType> Debug for SyntaxChildren<'a, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxChildren::One(val) =>
                f.debug_list()
                    .entries(vec![ val ])
                    .finish(),

            SyntaxChildren::Many(vals) => {
                let rules = vals
                    .iter()
                    .map(|child| child.rule);

                f.debug_list()
                    .entries(rules)
                    .finish()
            }
        }
    }
}

impl<'a, R> From<Pair<'a, R>> for SyntaxTree<'a, R>
where R: RuleType + DropRules<Rule = R>
{
    fn from(pair: Pair<'a, R>) -> Self {
        let rule = pair.as_rule();
        let providence = Providence { src: pair.as_str(), span: pair.as_span() };
        let skip_rules = rule.get_drop_rules();

        let mut inner_without_skip_rules = pair.into_inner()
            .filter(|pair| !(skip_rules.contains(&pair.as_rule())))
            .map(SyntaxTree::from);

        let children = match inner_without_skip_rules.next() {
            None => None,
            Some(first_child) => {
                match inner_without_skip_rules.next() {
                    None =>
                        Some(SyntaxChildren::One(first_child.into())),

                    Some(second_child) => {
                        let children = once(first_child)
                            .chain(once(second_child))
                            .chain(inner_without_skip_rules)
                            .collect();

                        Some(SyntaxChildren::Many(children))
                    }
                }
            }
        };

        SyntaxTree { rule, providence, children }
    }
}

impl<'a, R: RuleType> Debug for SyntaxTree<'a, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxTree { rule, providence, children: None } =>
                f.debug_struct("TreeLeaf")
                    .field("rule", rule)
                    .field("providence", providence)
                    .finish(),

            SyntaxTree { rule, providence, children: Some(children) } =>
                f.debug_struct("TreeNode")
                    .field("rule", rule)
                    .field("providence", providence)
                    .field("children", children)
                    .finish(),
        }
    }
}
