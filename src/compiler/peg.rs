use pest::iterators::{Pair, Pairs};

#[allow(missing_docs)]
#[derive(Parser)]
#[grammar = "grammars/vertex.pest"]
pub struct VertexLangParser;


/// Gets the next pair within the pairs iterator if the next element within the
/// iterator matches the indicated rule type. If it does not match, None is
/// returned and the iterator is not changed.
pub(super) fn get_rule<'a>(pair: &'a mut Pairs<Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    match pair.peek() {
        Some(p) => {
            if p.as_rule() == rule {
                pair.next(); // Skip since we just checked it.
                Some(p)
            } else {
                None
            }
        },
        None => None,
    }
}
