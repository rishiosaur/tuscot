#![allow(dead_code, unreachable_patterns)]

mod ast;
mod errors;
mod evaluator;
mod lexer;
mod objects;
mod parser;
mod token;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
