// use super::Context;
// use crate::{eval, parser::Parser, types::Statement};
// use anyhow::Result;

// fn solve_all(source: &str) -> Result<(Vec<String>)> {
//     let mut rules = Context::default();
//     let mut acc = Vec::new();

//     for stmt in Parser::from(source) {
//         match stmt? {
//             Statement::Rewrite(expr) => {
//                 acc.push(rules.rewrite(expr).to_string());
//             }
//             stmt => eval(stmt, &mut rules)?,
//         }
//     }
//     Ok(acc)
// }

// #[test]
// fn apply_all_reductions() {
//     let source = r"infix + 100
//                    let $A + 0 => $A
//                    let $A + $B => $B + $A
//                    rewrite (0 + 5) + 0";
//     assert_eq!(solve_all(source).unwrap(), vec!["5".to_string()]);
// }

// #[test]
// fn rewrite_is_deterministic() {
//     let source = r"infix + 100
//                    let $A + 0 => $A
//                    let $A + $B => $B + $A
//                    rewrite (0 + 5) + 0";
//     let first = solve_all(source).unwrap();
//     for _ in 0..20 {
//         assert_eq!(solve_all(source).unwrap(), first);
//     }
// }

// #[test]
// fn rewrite_applies_mapping() {
//     let source = r"let foo => bar
//                    rewrite foo";
//     assert_eq!(solve_all(source).unwrap(), vec!["bar".to_string()]);
// }

// #[test]
// fn literal_mappings() {
//     let source = r"let a => b
//                    rewrite a = b
//                    let b => c
//                    rewrite a = c";
//     assert_eq!(
//         solve_all(source).unwrap(),
//         vec!["b = b".to_string(), "c = c".to_string()]
//     );
// }

// #[test]
// fn mappings_are_unidirectional() {
//     let source = r"let a => b
//                    let a => c
//                    rewrite b = c";
//     assert_eq!(solve_all(source).unwrap(), vec!["b = c".to_string()]);
// }

// #[test]
// fn validate_facts() {
//     let source = r"let a
//                    let not b
//                    let a and (b and c)";
//     assert!(solve_all(source).is_err())
// }
