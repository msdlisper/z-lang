use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{pest::frame::ScopeStruct, util::SimpleError};

use super::{
    frame::{Frame, Scope, ValType},
    parse_ast::{parse_code, parse_file, AstNodeType},
};

fn main() {
    // 创建全局作用域
    let global_scope: Rc<RefCell<Scope>> = Rc::new(RefCell::new(Scope::Block(ScopeStruct {
        play_object: HashMap::new(),
        parent_scope: None,
    })));
    // 建立帧
    let first_frame = Frame {
        parent_frame: None,
        scope: global_scope,
    };
    let frame = Rc::new(RefCell::new(first_frame));
    let asts = parse_file();
    let mut res = ValType::Boolean(true);
    for ast in asts {
        res = match *eval(ast, frame.clone()).unwrap() {
            ValType::Number(n) => ValType::Number(n),
            // ValType::String(n) => ValType::String(n),
            _ => ValType::Boolean(true),
        }
    }
    println!("{:?}", res);
}

fn test_entry(code: String) -> ValType {
    // 创建全局作用域
    let global_scope: Rc<RefCell<Scope>> = Rc::new(RefCell::new(Scope::Block(ScopeStruct {
        play_object: HashMap::new(),
        parent_scope: None,
    })));
    // 建立帧
    let first_frame = Frame {
        parent_frame: None,
        scope: global_scope,
    };
    let frame = Rc::new(RefCell::new(first_frame));
    let asts = parse_code(code);
    let mut res = ValType::Boolean(true);
    for ast in asts {
        let eval_res = &*eval(ast, frame.clone()).unwrap();
        res = match eval_res {
            ValType::Number(n) => ValType::Number(*n),
            ValType::Boolean(n) => ValType::Boolean(*n),
            ValType::Closure {
                scope,
                block,
                name,
                args: _,
            } => ValType::String(format!("closure {}", name.clone())),
            // } => ValType::Closure { scope: scope.clone(), block: block.clone(), name: name.clone(), args: vec![] },
            // ValType::String(n) => ValType::String(n),
            _ => ValType::Boolean(true),
        }
    }
    res
}

fn eval(ast: AstNodeType, frame: Rc<RefCell<Frame>>) -> Result<Rc<ValType>, SimpleError> {
    let mut result: Option<Rc<ValType>> = None;

    match ast {
        AstNodeType::Statement { child } => {
            for exp in *child {
                result = Some(eval(exp, frame.clone())?);
            }
        }

        AstNodeType::IntLiteral(val) => {
            result = Some(Rc::new(ValType::Number(val)));
        }

        AstNodeType::AdditiveExp { child } => {
            let mut res: i32 = 0;
            for child_item in child {
                let val = &*eval(child_item, frame.clone())?;
                let eval_res = match val {
                    ValType::Number(n) => n,
                    ValType::Closure {
                        scope,
                        block,
                        name,
                        args,
                    } => {
                        return Ok(Rc::new(ValType::Closure {
                            scope: scope.clone(),
                            block: block.clone(),
                            name: name.clone(),
                            args: vec![],
                        }));
                    }
                    _ => return Err(SimpleError::Calc("Expected number".to_string())),
                };
                res = res + eval_res;
            }
            result = Some(Rc::new(ValType::Number(res)))
        }

        AstNodeType::MulitiExp { child } => {
            let mut start: i32 = 1;
            for child_item in child {
                let val = &*eval(child_item, frame.clone())?;
                let eval_res = match val {
                    ValType::Number(n) => n,
                    ValType::Closure {
                        scope,
                        block,
                        name,
                        args,
                    } => {
                        return Ok(Rc::new(ValType::Closure {
                            scope: scope.clone(),
                            block: block.clone(),
                            name: name.clone(),
                            args: vec![],
                        }));
                    }
                    _ => return Err(SimpleError::Calc("Expected number".to_string())),
                };
                start = start * eval_res;
            }
            result = Some(Rc::new(ValType::Number(start)));
        }

        AstNodeType::Identifier { ident } => {
            let var: String = ident.clone();
            let fr = frame.borrow_mut();
            if fr.contains_key(&var) {
                match fr.get(&var) {
                    Some(res) => {
                        result = Some(res);
                    }
                    None => {
                        return Err(SimpleError::Calc((var + " key没有值").to_string()));
                    }
                }
            } else {
                return Err(SimpleError::Calc((var + " key没有申明").to_string()));
            }
        }

        AstNodeType::JudgeExp { left, right, judge } => {
            let left_val = eval(*left, frame.clone())?;
            let right_val = eval(*right, frame.clone())?;

            match judge.as_str() {
                ">" => {
                    let res = *left_val > *right_val;
                    result = Some(Rc::new(ValType::Boolean(res)))
                }
                "<" => {
                    let res = *left_val < *right_val;
                    result = Some(Rc::new(ValType::Boolean(res)))
                }
                ">=" => {
                    let res = *left_val >= *right_val;
                    result = Some(Rc::new(ValType::Boolean(res)))
                }
                "<=" => {
                    let res = *left_val <= *right_val;
                    result = Some(Rc::new(ValType::Boolean(res)))
                }
                "==" => {
                    let res = *left_val == *right_val;
                    result = Some(Rc::new(ValType::Boolean(res)))
                }
                _ => {
                    return Err(SimpleError::Calc((judge + " 运算符没有实现").to_string()));
                }
            }
        }

        AstNodeType::Declaration {
            declare_type,
            identifier,
            additive,
        } => {
            let var: String = identifier.clone();
            let var_value = eval(*additive.unwrap(), frame.clone())?;
            frame.borrow_mut().set(var, var_value, true);
            result = Some(Rc::new(ValType::Boolean(true)));
        }

        AstNodeType::AssignmentStatement { ident, additive } => {
            let var: String = ident.clone();
            let mut is_contains = false;
            {
                // 避免同一时间有可变借用和不可变节用
                is_contains = frame.borrow().contains_key(&var);
            }
            match is_contains {
                true => {
                    let var_value = eval(*additive, frame.clone())?;
                    let mut fr = frame.borrow_mut();
                    fr.set(var, var_value.clone(), false);
                    result = Some(var_value);
                }
                false => {
                    return Err(SimpleError::Calc((var + " key没有申明").to_string()));
                }
            }
        }

        AstNodeType::IfStatement {
            judge_stat,
            if_stat,
            else_stat,
        } => {
            let judge: bool = match eval(*judge_stat, frame.clone())?.as_ref() {
                ValType::Boolean(b) => *b,
                val => unreachable!("judge返回的不是bool: {:?}", val),
            };
            if judge {
                eval(*if_stat, frame.clone())?;
            } else if else_stat.is_some() {
                let else_statement = else_stat.unwrap();
                eval(*else_statement, frame.clone())?;
            }
            result = Some(Rc::new(ValType::Boolean(true)));
        }

        AstNodeType::ReturnExp { exp } => {
            match exp {
                Some(val) => {
                    result = Some(eval(*val, frame.clone())?);
                },
                None => {
                    result = Some(Rc::new(ValType::Boolean(false)));
                }
            }
        }

        AstNodeType::Block { statements } => {
            // 创建新的scope
            {
                let new_scope = Rc::new(RefCell::new(Scope::Block(ScopeStruct {
                    play_object: HashMap::new(),
                    parent_scope: None,
                })));
                let mut fr = frame.borrow_mut();
                fr.create_scope(new_scope);
            }

            let val = match statements {
                Some(stats) => {
                    let mut val = Rc::new(ValType::Boolean(false));
                    for stat in stats {
                        val = eval(stat, frame.clone())?;
                    }
                    val
                }
                None => {
                    Rc::new(ValType::Boolean(false))
                }
            };
            result = Some(val);
            // 销毁作用域
            let mut fr = frame.borrow_mut();
            fr.drop_scope();
        }

        AstNodeType::FnCall {
            identifier,
            // 参数列表
            argu_list,
        } => {
            let fn_ident = &*eval(*identifier, frame.clone())?;
            match fn_ident {
                ValType::Closure {
                    scope,
                    block,
                    name,
                    args,
                } => {
                    // let mut params = Vec::new();
                    // for arg in argu_list {
                    //     params.push(eval(arg, frame.clone())?);
                    // }

                    // 创建新的frame
                    let new_frame = Frame {
                        scope: scope.clone(),
                        parent_frame: Some(frame.clone()),
                    };

                    /**
                     * TODO
                     * 这里的实现, 需要入参表达式在新的作用域里执行
                     */
                    //  将入参在新的作用域声明(补充block的Declaration节点)
                    let need_insert_declare = argu_list.len() > 0;
                    if need_insert_declare {
                        // 生成statements
                        let mut statements = vec![];
                        for (index, arg) in args.iter().enumerate() {
                            // 得到declare节点
                            // println!("arg: {:#?}", arg);
                            // println!("params: {:#?}", argu_list[index]);
                            match arg {
                                AstNodeType::Declaration {
                                    declare_type,
                                    identifier,
                                    additive: _,
                                } => {
                                    // TODO: additive可能有默认值, 然后parame缺省的情况
                                    let new_declare = AstNodeType::Declaration {
                                        declare_type: declare_type.clone(),
                                        identifier: identifier.clone(),
                                        additive: Some(Box::new(argu_list[index].clone())),
                                    };
                                    statements.push(AstNodeType::Statement {
                                        child: Box::new(vec![new_declare]),
                                    });
                                }
                                _ => {}
                            }
                        }

                        let mut declare_ast = AstNodeType::Block {
                            statements: Some(statements),
                        };
                        let block_statements = match &**block {
                            AstNodeType::Block { statements } => statements.clone().unwrap(),
                            _ => panic!("block_statements is not block"),
                        };

                        // 判断需要合并declare和fncall里面的
                        match declare_ast {
                            AstNodeType::Block { ref mut statements } => {
                                let mut inner_stats = statements.as_mut().unwrap();
                                inner_stats.extend(block_statements);
                                // println!("inner_stats: {:#?}", inner_stats);
                            }
                            _ => {}
                        };

                        // 执行block (会创建新的作用域)
                        let res = eval(declare_ast, Rc::new(RefCell::new(new_frame)));
                        // println!("block执行结果: {:#?}", res);
                        result = Some(res?);
                    } else {
                        // 执行block
                        // TODO这里的clone()比较消耗, clone了整个树
                        let res = eval((**block).clone(), Rc::new(RefCell::new(new_frame)));
                        result = Some(res?);
                    }

                    // 丢掉这个frame(new_frame会默认丢掉)
                    frame.borrow_mut().parent_frame = None;
                    // 执行完成后将return语句的值给result
                    // TODO
                }
                _ => {
                    // 函数调用错误
                    return Err(SimpleError::Calc("函数调用出错".into()));
                }
            }
            // result = Some(Rc::new(ValType::Boolean(true)));
        }

        AstNodeType::FnDeclaration {
            return_type,
            identifier,
            argu_list,
            block,
        } => {
            let mut fr = frame.borrow_mut();
            let closure = Rc::new(ValType::Closure {
                scope: fr.scope.clone(),
                block: Rc::new(*block),
                name: identifier.clone(),
                args: argu_list,
            });
            fr.set(identifier, closure, true);
            result = Some(Rc::new(ValType::Boolean(true)));
        }

        _ => {
            println!("{:?}", ast);
            return Err(SimpleError::Calc((" 没处理该表达式").to_string()));
        }
    }
    Ok(result.unwrap())
}

#[cfg(test)]
mod tests;
