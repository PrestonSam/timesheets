use generic_utils::SyntaxTree;
use pest::iterators::Pairs;
use pack_trees::{unpack_only_tree, TokenPacker};

pub use packer::*;
pub use generic_utils::{DropRules, PackingError};
pub use generic_model::PackingErrorVariant;

use crate::parser::Rule;

mod generic_utils;
mod pack_trees;
mod generic_model;
mod packer;


pub fn pack(pairs: Pairs<'_, Rule>) -> Result<Weeks, PackingError<PackingErrorVariant<Rule>, Rule>> {
    let trees: Vec<_> = pairs.map(|p| SyntaxTree::<'_, Rule>::from(p))
        .collect();

    unpack_only_tree(trees)
        .and_then(Body::pack)
        .map(|body| body.0)
}
