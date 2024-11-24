#[derive(Debug)]
pub enum SimpleError {
  Lex(String),
  Ast(String),
  Calc(String),
}