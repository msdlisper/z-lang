use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest::Parser;
use std::io::{self, BufRead};
use std::{env, fs, vec};

use super::frame::ValType;

mod tests;

#[derive(pest_derive::Parser)]
#[grammar = "calc.pest"]
pub struct CalculatorParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            // .op(Op::prefix(unary_minus))
    };
}

// #[derive(Debug)]
// pub enum Expr {
//     Integer(i32),
//     UnaryMinus(Box<Expr>),
//     BinOp {
//         lhs: Box<Expr>,
//         op: Op,
//         rhs: Box<Expr>,
//     },
// }

#[derive(Debug, Clone)]
pub enum DeclareType {
    Int,
    String,
    FnType {
        return_type: Option<Box<DeclareType>>,
        // 这里只能是declare_stat, 比如int , fn int ()
        argu_list: Box<Vec<DeclareType>>,
    },
}

#[derive(Debug, Clone)]
pub enum AstNodeType {
    Statement {
        child: Box<Vec<AstNodeType>>,
    },
    IntLiteral(i32),
    AdditiveExp {
        // mult
        child: Vec<AstNodeType>,
    },
    MulitiExp {
        // atom
        child: Vec<AstNodeType>,
    },
    JudgeExp {
        // mult
        left: Box<AstNodeType>,
        right: Box<AstNodeType>,
        judge: String,
    },
    IfStatement {
        judge_stat: Box<AstNodeType>,
        // blocks
        if_stat: Box<AstNodeType>,
        else_stat: Option<Box<AstNodeType>>,
    },
    Block {
        statements: Option<Vec<AstNodeType>>,
    },
    Identifier {
        ident: String,
    },
    AssignmentStatement {
        // I属性, 从申明中找到
        ident: String,
        // S属性, 下级节点推导, check
        additive: Box<AstNodeType>,
    },
    // 函数体定义
    FnDeclaration {
        return_type: Option<Box<DeclareType>>,
        identifier: String,
        // Declaration, 比如int a, fn int () b
        argu_list: Vec<AstNodeType>,
        // 这里只能是AstNodeType::Block
        block: Box<AstNodeType>,
    },
    // 函数调用
    FnCall {
        // AstNodeType::Identifier
        identifier: Box<AstNodeType>,
        argu_list: Vec<AstNodeType>,
    },
    // 普通声明
    Declaration {
        // S属性 由下级节点推导
        declare_type: DeclareType,
        // I属性, 根节点继承下来
        identifier: String,
        // S属性, 下级节点推导 check
        additive: Option<Box<AstNodeType>>,
    },
    ReturnExp {
        // return_stat = {return ~ additive | judge_stat | fn_declare }
        exp: Option<Box<AstNodeType>>,
    },
}

fn parse_add(pair: Pair<Rule>) -> AstNodeType {
    let mut ast: Vec<AstNodeType> = vec![];

    // 将mul拿出来相加
    let muls = pair.into_inner();
    for mul in muls {
        match mul.as_rule() {
            Rule::multi => {
                let multi_node = parse_mul(mul);
                ast.push(multi_node);
            }
            Rule::add => {
                // nothing
            }
            rule => {
                unreachable!("Expr::parse expected multi operation, found {:?}", rule)
            }
        }
    }

    AstNodeType::AdditiveExp {
        child: ast,
    }
}

fn parse_atom(pair: Pair<Rule>) -> AstNodeType {
    match pair.as_rule() {
        Rule::ident => AstNodeType::Identifier {
            ident: pair.as_str().into(),
        },
        Rule::integer => AstNodeType::IntLiteral(pair.as_str().parse().unwrap()),
        Rule::fn_call => parse_fn_call(pair),
        rule => {
            unreachable!("Expr::parse expected atom operation, found {:?}", rule);
        }

    }
}
fn parse_mul(pair: Pair<Rule>) -> AstNodeType {
    let mut ast: Vec<AstNodeType> = vec![];

    let atoms = pair.into_inner();

    for atom in atoms {
        if atom.as_rule() != Rule::multiply {
            ast.push(parse_atom(atom));
        }
    }
    AstNodeType::MulitiExp {
        child: ast,
    }
}

pub fn parse_simple(pairs: Pairs<Rule>) -> Vec<AstNodeType> {
    let mut ast: Vec<AstNodeType> = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::statement => {
                let inner = pair.into_inner();

                let parsed_child = parse_stat(inner);

                let state_node = AstNodeType::Statement {
                    child: Box::new(parsed_child),
                };
                ast.push(state_node);
            }
            _ => {
                println!("{:?} 不是statement", pair.as_rule());
            }
        }
    }
    ast
}

/**
 * 解析各种情况
 */
fn parse(pair: Pair<Rule>) -> Option<AstNodeType> {
    match pair.as_rule() {
        Rule::assi_stat => {
            let mut child_inner = pair.into_inner();
            let ident = child_inner.next().unwrap();
            child_inner.next();
            // 加法
            let additive = child_inner.next().unwrap();
            let ass_node = AstNodeType::AssignmentStatement {
                ident: ident.as_str().into(),
                additive: Box::new(parse_add(additive)),
            };
            return Some(ass_node);
        }
        Rule::declare_stat => {
            // 判断声明的类型, 生成Declaration节点
            let mut child_inner = pair.into_inner();
            let declare_type = child_inner.next().unwrap();
            let mut type_inner = declare_type.clone().into_inner();
            let some_type = type_inner.next().unwrap();

            match some_type.as_rule() {
                Rule::int => {
                    let ident = child_inner.next().unwrap();
                    child_inner.next();
                    // 加法
                    let expr = child_inner.next();
                    let declare_node = AstNodeType::Declaration {
                        declare_type: DeclareType::Int,
                        identifier: ident.as_str().into(),
                        additive: match expr {
                            Some(exp) => Some(Box::new(parse_add(exp))),
                            None => None,
                        },
                    };
                    return Some(declare_node);
                }
                Rule::fn_type => {
                    let fn_type_node = parse_type(declare_type);
                    let ident = child_inner.next().unwrap();
                    child_inner.next();
                    // 表达式
                    let expr = child_inner.next();
                    match fn_type_node {
                        Some(fn_typed) => {
                            let fn_declare_node = AstNodeType::Declaration {
                                declare_type: fn_typed,
                                identifier: ident.as_str().into(),
                                additive: match expr {
                                    Some(exp) => Some(Box::new(parse_add(exp))),
                                    None => None,
                                },
                            };
                            return Some(fn_declare_node);
                        }
                        None => return None,
                    }
                }
                _ => {
                    return None;
                }
            }
        }
        Rule::additive => return Some(parse_add(pair)),
        Rule::judge_stat => return Some(parse_judge(pair)),
        Rule::if_statement => return Some(parse_if(pair)),
        Rule::fn_declare => return Some(parse_fn_decare(pair)),
        Rule::return_stat => {
            let mut child_inner = pair.into_inner();
            child_inner.next();
            // 找到return后面的表达式
            // additive | judge_stat | fn_declare
            let expr = parse(child_inner.next().unwrap());
            let return_node = AstNodeType::ReturnExp {
                exp: match expr {
                    Some(node) => Some(Box::new(node)),
                    None => None,
                },
            };
            return Some(return_node);
        }
        r => {
            //nothing
            None
        }
    }
}

fn parse_type_or_void(pair: Pair<Rule>) -> Option<Box<DeclareType>> {
    let type_or_void_inner = pair.into_inner().peek().unwrap();
    let what_type = match type_or_void_inner.as_rule() {
        Rule::void => None,
        _ => match parse_type(type_or_void_inner) {
            Some(node) => Some(Box::new(node)),
            None => None,
        },
    };
    return what_type;
}

/**
 * 解析函数声明
 */
fn parse_fn_decare(pair: Pair<Rule>) -> AstNodeType {
    let mut child_inner = pair.into_inner();

    let return_type = parse_type_or_void(child_inner.next().unwrap());

    let identifier: String = child_inner.next().unwrap().as_str().into();

    let argu_list = child_inner.next().unwrap();
    let mut argu_list_inner = argu_list.into_inner();
    let mut params = Vec::new();
    while let Some(declare_stat) = argu_list_inner.next() {
        params.push(parse(declare_stat).unwrap());
    }

    let body = child_inner.next().unwrap();
    let body_node = parse_block(body);
    let fn_declare_node = AstNodeType::FnDeclaration {
        return_type,
        identifier,
        argu_list: params,
        block: Box::new(body_node),
    };

    fn_declare_node
}

fn parse_fn_call(pair: Pair<Rule>) -> AstNodeType {
    let mut child_inner = pair.into_inner();
    let identifier: String = child_inner.next().unwrap().as_str().into();
    let mut argu_list = vec![];

    while let Some(argu) = child_inner.next() {
        argu_list.push(parse_add(argu));
    };
    AstNodeType::FnCall { identifier: Box::new(AstNodeType::Identifier { ident: identifier }), argu_list, }
    
}

/**
 * 解析各种type
 * 生成AstNodeType
 */
fn parse_type(pair: Pair<Rule>) -> Option<DeclareType> {
    let mut child_inner = pair.into_inner();
    let type_node = child_inner.next().unwrap();
    match type_node.as_rule() {
        Rule::int => {
            let type_node = DeclareType::Int;
            Some(type_node)
        }
        Rule::fn_type => {
            let mut fn_inner = type_node.into_inner();
            // 跳过functioon
            fn_inner.next();
            let type_or_void = fn_inner.next().unwrap();
            let return_type = parse_type_or_void(type_or_void);
            // parse type_or_void;
            let type_list = fn_inner.next().unwrap();
            let mut list_vec = vec![];
            for item in type_list.into_inner() {
                match parse_type(item) {
                    Some(node) => {
                        list_vec.push(node);
                    }
                    None => {
                        panic!("item节点不是type{:?}", fn_inner);
                    }
                }
            }

            let fn_typed = DeclareType::FnType {
                return_type: return_type,
                argu_list: Box::new(list_vec),
            };
            Some(fn_typed)
        }
        _ => None,
    }
}

fn parse_stat(childs: Pairs<Rule>) -> Vec<AstNodeType> {
    let mut ast: Vec<AstNodeType> = vec![];

    for child in childs {
        match parse(child) {
            Some(node) => ast.push(node),
            None => {
                // nothing
            }
        }
    }

    ast
}

fn parse_judge(pair: Pair<Rule>) -> AstNodeType {
    let mut child_inner = pair.into_inner();
    // 找到左边和右边
    let left = child_inner.next().unwrap();
    let judge = child_inner.next().unwrap();
    let right = child_inner.next().unwrap();
    AstNodeType::JudgeExp {
        left: Box::new(parse_add(left)),
        right: Box::new(parse_add(right)),
        judge: judge.as_str().into(),
    }
}

fn parse_block(pair: Pair<Rule>) -> AstNodeType {
    let blocks_inner = pair.into_inner();
    // 找到左边和右边
    if blocks_inner.len() == 0 {
        return AstNodeType::Block { statements: None };
    }

    let mut ast: Vec<AstNodeType> = vec![];
    // 简单校验
    for block in blocks_inner.clone() {
        match block.as_rule() {
            Rule::statement => {
                let inner = block.into_inner();

                let parsed_child = parse_stat(inner);

                let state_node = AstNodeType::Statement {
                    child: Box::new(parsed_child),
                };
                ast.push(state_node);
            }
            rule => unreachable!("Expr::parse expected statement, found {:?}", rule),
        }
    }

    AstNodeType::Block {
        statements: Some(ast),
    }
}

fn parse_if(pair: Pair<Rule>) -> AstNodeType {
    let mut if_inner = pair.into_inner();
    let inner_len = if_inner.len();
    if_inner.next();
    let judge_stat = if_inner.next().unwrap();
    let if_block = if_inner.next().unwrap();

    if (inner_len == 3) {
        return AstNodeType::IfStatement {
            judge_stat: Box::new(parse_judge(judge_stat)),
            if_stat: Box::new(parse_block(if_block)),
            else_stat: None,
        };
    }

    // 跳过else
    if_inner.next();
    let else_block = if_inner.next().unwrap();
    let else_node = Some(Box::new(parse_block(else_block)));

    AstNodeType::IfStatement {
        judge_stat: Box::new(parse_judge(judge_stat)),
        if_stat: Box::new(parse_block(if_block)),
        else_stat: else_node,
    }
}

pub fn parse_code(code: String) -> Vec<AstNodeType> {
    match CalculatorParser::parse(Rule::equation, &code) {
        Ok(pairs) => {
            // println!("Parsed: {:#?}", pairs);
            let res = parse_simple(pairs);
            println!(
                "Parsed: {:#?}",
                // inner of expr
                res // pairs
            );
            return res;
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
            return vec![];
        }
    }
}

pub fn parse_file() -> Vec<AstNodeType> {
    let current_dir = env::current_dir().expect("无法获取当前工作目录");
    // 构建文件路径
    let file_path = current_dir.join("src/pest/calc.sc");

    let unparsed_file = fs::read_to_string(file_path).expect("cannot read file");

    // println!("{:?}", unparsed_file);

    match CalculatorParser::parse(Rule::equation, &unparsed_file) {
        Ok(mut pairs) => {
            let res = parse_simple(pairs);
            println!(
                "Parsed: {:#?}",
                // inner of expr
                res // pairs
            );
            return res;
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
            return vec![];
        }
    }
}
