use pest::RuleType;

use crate::parser::Rule;

use super::{generic_model::{PackingErrorVariant, RuleAndProvidence, PackingResult}, generic_utils::{PackingError, SyntaxTree}};


pub trait HasRule
where Self: Sized
{
    type Rule: RuleType;

    fn get_rule() -> Self::Rule;
}

pub trait TokenPacker: HasRule
where
    Self: Sized,
{
    fn is_packable(tree: &SyntaxTree<'_, Self::Rule>) -> bool {
        tree.rule == Self::get_rule()
    }

    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>>;
}

pub trait TokenRepacker: HasRule
{
    type Packer: TokenPacker;

    fn repack(packer: Self::Packer) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>>;
}

impl<T, P> TokenPacker for T
where
    T: TokenRepacker<Packer = P>,
    P: TokenPacker<Rule = T::Rule>,
{
    fn pack(tree: SyntaxTree<'_, Self::Rule>) -> Result<Self, PackingError<PackingErrorVariant<Self::Rule>, Self::Rule>> {
        <T as TokenRepacker>::repack(P::pack(tree)?)
    }
}


fn into_packing_error<R: RuleType>(trees: (Option<SyntaxTree<R>>, Option<SyntaxTree<R>>), expected_rule: Option<R>) -> PackingError<PackingErrorVariant<R>, R> {
    let variant = match trees {
        ( Some(SyntaxTree { rule: found_rule, providence, .. })
        , None
        ) =>
            PackingErrorVariant::WrongRulesSeq {
                found_rules: vec![ RuleAndProvidence(found_rule, providence.as_string()) ],
                expected_rules: expected_rule.map(|rule| vec![ rule ]),
            },

        ( Some(SyntaxTree { rule: found_rule_1, providence: providence_1, .. })
        , Some(SyntaxTree { rule: found_rule_2, providence: providence_2, .. })
        ) =>
            PackingErrorVariant::TooManyRules {
                present_rules: vec![
                    RuleAndProvidence(found_rule_1, providence_1.as_string()),
                    RuleAndProvidence(found_rule_2, providence_2.as_string()),
                ],
                expected_rules: expected_rule.map(|rule| vec![ rule ]),
                expected_count: 1
            },

        (None, _) =>
            PackingErrorVariant::NoChildrenFound {
                expected_rules: expected_rule.map(|rule| vec![ rule ]),
            },
    };

    PackingError::new(variant)
}

fn into_packing_error_2<R: RuleType>(
    trees: (Option<SyntaxTree<R>>, Option<SyntaxTree<R>>, Option<SyntaxTree<R>>),
    expected_rules: Option<Vec<R>>
) -> PackingError<PackingErrorVariant<R>, R>
{
    let variant = match trees {
        ( Some(SyntaxTree { rule: found_rule_1, providence: providence_1, .. })
        , Some(SyntaxTree { rule: found_rule_2, providence: providence_2, .. })
        , None
        ) =>
            PackingErrorVariant::WrongRulesSeq {
                found_rules: vec![
                    RuleAndProvidence(found_rule_1, providence_1.as_string()),
                    RuleAndProvidence(found_rule_2, providence_2.as_string()),
                ],
                expected_rules,
            },

        ( Some(SyntaxTree { rule: found_rule_1, providence: providence_1, .. })
        , Some(SyntaxTree { rule: found_rule_2, providence: providence_2, .. })
        , Some(SyntaxTree { rule: found_rule_3, providence: providence_3, .. })
        ) =>
            PackingErrorVariant::TooManyRules {
                present_rules: vec![
                    RuleAndProvidence(found_rule_1, providence_1.as_string()),
                    RuleAndProvidence(found_rule_2, providence_2.as_string()),
                    RuleAndProvidence(found_rule_3, providence_3.as_string()),
                ],
                expected_rules,
                expected_count: 2
            },

        ( Some(SyntaxTree { rule: found_rule_1, providence: providence_1, .. })
        , None
        , _
        ) =>
            PackingErrorVariant::TooFewRules {
                present_rules: vec![ RuleAndProvidence(found_rule_1, providence_1.as_string()), ],
                expected_rules,
                expected_count: 2
            },

        (None, _, _) =>
            PackingErrorVariant::NoChildrenFound {
                expected_rules,
            },
    };

    PackingError::new(variant)
}

pub fn make_wrong_rules_alt_error<R: RuleType>(tree: SyntaxTree<R>, expected_rules: Vec<R>) -> PackingError<PackingErrorVariant<R>, R> {
    let found_rule = RuleAndProvidence(tree.rule, tree.providence.as_string());
    let variant = PackingErrorVariant::WrongRulesAlt { found_rule, expected_rules };

    PackingError::new(variant)
}

pub fn unpack_only_tree<R: RuleType>(trees: Vec<SyntaxTree<R>>) -> Result<SyntaxTree<R>, PackingError<PackingErrorVariant<R>, R>> {
    let mut trees = trees.into_iter();

    match (trees.next(), trees.next()) {
        (Some(tree @ SyntaxTree { .. }), None) =>
            Ok(tree),

        _ =>
            todo!("Produce error complaining wrong number of children"),
    }
}

pub fn get_tree_src_string<R: RuleType>(tree: SyntaxTree<R>, expected_rule: R) -> Result<String, PackingError<PackingErrorVariant<R>, R>> {
    match tree {
        SyntaxTree { rule, providence, children: None } if rule == expected_rule =>
            Ok(providence.as_trimmed_string()),
        
        _ =>
            todo!()
    }.with_rule(expected_rule)
}

pub fn get_only_tree_child<R: RuleType>(tree: SyntaxTree<R>, expected_rule: R) -> Result<SyntaxTree<R>, PackingError<PackingErrorVariant<R>, R>> {
    match tree {
        SyntaxTree { rule, children: Some(children), .. } if rule == expected_rule =>
            unpack_only_tree(children.get_values()),

        _ =>
            todo!()
    }.with_rule(expected_rule)
}

pub fn get_tree_children<R: RuleType>(tree: SyntaxTree<R>, expected_rule: R) -> Result<Vec<SyntaxTree<R>>, PackingError<PackingErrorVariant<R>, R>> {
    match tree {
        SyntaxTree { rule, children: Some(children), .. } if rule == expected_rule =>
            Ok(children.get_values()),
        
        _ =>
            todo!()
    }.with_rule(expected_rule)
}

fn pack_each_tree<P>(trees: Vec<SyntaxTree<P::Rule>>) -> Result<Vec<P>, PackingError<PackingErrorVariant<P::Rule>, P::Rule>>
where P : TokenPacker
{
    trees.into_iter()
        .map(|tree| P::pack(tree))
        .collect::<Result<_, _>>()
}

fn pack_1_tree<P>(trees: Vec<SyntaxTree<P::Rule>>) -> Result<P, PackingError<PackingErrorVariant<P::Rule>, P::Rule>>
where P: TokenPacker {
    let mut trees = trees.into_iter();

    match (trees.next(), trees.next()) {
        ( Some(child @ SyntaxTree { .. })
        , None
        ) => {
            let packed = P::pack(child)?;

            Ok(packed)
        },

        trees =>
            Err(into_packing_error(trees, None))
    }
}

fn pack_2_trees<P1, P2, R>(trees: Vec<SyntaxTree<R>>) -> Result<(P1, P2), PackingError<PackingErrorVariant<R>, R>>
where
    R: RuleType,
    P1: TokenPacker<Rule = R>,
    P2: TokenPacker<Rule = R>
{
    let mut trees = trees.into_iter();

    match (trees.next(), trees.next(), trees.next()) {
        ( Some(child_1 @ SyntaxTree { .. })
        , Some(child_2 @ SyntaxTree { .. })
        , None
        ) => {
            let packed_1 = P1::pack(child_1)?;
            let packed_2 = P2::pack(child_2)?;

            Ok((packed_1, packed_2))
        },

        trees =>
            Err(into_packing_error_2(trees, None))
    }
}

pub fn pack_next_tree<'a, I, P, R: RuleType>(trees: &mut I) -> Result<P, PackingError<PackingErrorVariant<R>, P::Rule>>
where
    I: Iterator<Item = SyntaxTree<'a, P::Rule>>,
    P: TokenPacker<Rule = R>,
{
    match trees.next() {
        Some(child @ SyntaxTree { .. }) =>
            P::pack(child),

        None =>
            todo!("Produce error commenting that you're surprised")
    }
}

pub fn get_x_trees<const N: usize, R: RuleType>(trees: Vec<SyntaxTree<R>>) -> Result<[SyntaxTree<R>; N], PackingError<PackingErrorVariant<R>, R>> {
    trees.try_into()
        .map_err(|_| todo!("wrap / repackage error"))
}

pub fn ensure_no_more_trees<'a, I, R: RuleType>(mut trees: I) -> Result<(), PackingError<PackingErrorVariant<R>, R>>
where
    I: Iterator<Item = SyntaxTree<'a, R>>
{
    match trees.next() {
        None =>
            Ok(()),
        
        _ =>
            todo!("Produce error saying there are too many rules")
    }
}

fn pack_maybe_2_trees<P1, P2, R>(trees: Vec<SyntaxTree<R>>) -> Result<(P1, Option<P2>), PackingError<PackingErrorVariant<R>, R>>
where
    R: RuleType,
    P1: TokenPacker<Rule = R>,
    P2: TokenPacker<Rule = R>,
{
    let mut trees = trees.into_iter();

    match (trees.next(), trees.next(), trees.next()) {
        ( Some(child_1 @ SyntaxTree { .. })
        , Some(child_2 @ SyntaxTree { .. })
        , None
        ) => {
            let packed_1 = P1::pack(child_1)?;
            let packed_2 = P2::pack(child_2)?;

            Ok((packed_1, Some(packed_2)))
        },

        ( Some(child_1 @ SyntaxTree { .. })
        , None
        , None
        ) => {
            let packed_1 = P1::pack(child_1)?;

            Ok((packed_1, None))
        },

        trees =>
            Err(into_packing_error_2(trees, None))
    }
}

pub fn unpack_tree_pack_each_child<P>(tree: SyntaxTree<P::Rule>, expected_rule: P::Rule) -> Result<Vec<P>, PackingError<PackingErrorVariant<P::Rule>, P::Rule>>
where P : TokenPacker
{
    get_tree_children(tree, expected_rule)
        .and_then(pack_each_tree)
}

pub fn unpack_tree_pack_1_child<P>(tree: SyntaxTree<P::Rule>, expected_rule: P::Rule) -> Result<P, PackingError<PackingErrorVariant<P::Rule>, P::Rule>> where P: TokenPacker {
    get_tree_children(tree, expected_rule)
        .and_then(pack_1_tree)
}

pub fn unpack_tree_pack_2_children<P1, P2, R>(tree: SyntaxTree<R>, expected_rule: R) -> Result<(P1, P2), PackingError<PackingErrorVariant<R>, R>>
where
    R: RuleType,
    P1: TokenPacker<Rule = R>,
    P2: TokenPacker<Rule = R>
{
    get_tree_children(tree, expected_rule)
        .and_then(pack_2_trees)
}

pub fn unpack_tree_pack_maybe_2_children<P1, P2, R>(tree: SyntaxTree<R>, expected_rule: R) -> Result<(P1, Option<P2>), PackingError<PackingErrorVariant<R>, R>>
where
    R: RuleType,
    P1: TokenPacker<Rule = R>,
    P2: TokenPacker<Rule = R>
{
    get_tree_children(tree, expected_rule)
        .and_then(pack_maybe_2_trees)
}
