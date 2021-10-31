use crate::{Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{EntryHolder, LetIn, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

/// empty let-in found
#[lint(
    name = "empty let-in",
    note = "Useless let-in expression",
    code = 2,
    match_with = SyntaxKind::NODE_LET_IN
)]
struct EmptyLetIn;

impl Rule for EmptyLetIn {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(let_in_expr) = LetIn::cast(node.clone());
            let entries = let_in_expr.entries();
            let inherits = let_in_expr.inherits();

            if entries.count() == 0;
            if inherits.count() == 0;

            if let Some(body) = let_in_expr.body();
            then {
                let at = node.text_range();
                let replacement = body;
                let message = "This let-in expression has no entries";
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
