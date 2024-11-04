use pest::iterators::Pairs;

use token_packer::{generic_model::PackingErrorVariant, generic_utils::{PackingError, SyntaxTree}, pack_trees::{unpack_only_tree, TokenPacker}};
use crate::parser::Rule;

pub use packer::*;

mod packer;


pub fn pack(pairs: Pairs<'_, Rule>) -> Result<Weeks, PackingError<PackingErrorVariant<Rule>, Rule>> {
    let trees: Vec<_> = pairs.map(|p| SyntaxTree::<'_, Rule>::from(p))
        .collect();

    unpack_only_tree(trees)
        .and_then(Body::pack)
        .map(|body| body.0)
}
