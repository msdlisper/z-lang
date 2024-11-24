use pest::Parser;
use pest_derive::Parser;

#[allow(implied_bounds_entailment)]
mod parse_ast;

mod frame;
mod slick_script;



#[cfg(test)]
mod tests {
    use std::{env, fs};

    use super::*;

}
