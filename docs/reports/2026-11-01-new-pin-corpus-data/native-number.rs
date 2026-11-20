use unlaxer_runtime::{Expr, Rule, parse_detailed};
fn main() {
    for (input, prefix, suffix) in [("n1e+;", "n", ";"), ("neg(1e+)", "neg(", ")")] {
        let rules = [Rule { name:"Root", expression:Expr::Sequence(vec![Expr::Literal(prefix),Expr::Number,Expr::Literal(suffix)]) }];
        println!("{}\t{}", input, parse_detailed(&rules, 0, false, input).unwrap_err().canonical_json());
    }
}
