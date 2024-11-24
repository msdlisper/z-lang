use super::*;


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fn_call() {
        let binding = String::from("a(b,c(f),99)");
        let pairs = CalculatorParser::parse(Rule::fn_call, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_fn_call(p);
        println!("{:#?}", res);

        match res {
            AstNodeType::FnCall {
                identifier,
                argu_list,
            } => {
                assert_eq!(argu_list.len(), 3);
            }
            _ => {}
        }
    }

    #[test]
    // 各种类型
    fn test_type() {
        let binding = String::from("fn int (int, fn void ())");
        let pairs = CalculatorParser::parse(Rule::typed, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_type(p);
        // println!("{:#?}", res);
        match res.unwrap() {
            DeclareType::FnType { return_type, argu_list } => {
                assert_eq!(return_type.is_none(), false);
                assert_eq!(argu_list.len(), 2);
            },
            _ => {
                panic!("error");
            }
        }

    }

    #[test]
    //方法申明
    fn test_fn_declere() {
        let binding = String::from("void a (int c, int f = 9 )  {return c;}");
        let pairs = CalculatorParser::parse(Rule::fn_declare, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_fn_decare(p);
        // println!("{:#?}", res);
        match res {
          AstNodeType::FnDeclaration { return_type, identifier, argu_list, block } => {
              assert_eq!(return_type.is_none(), true);
              assert_eq!(identifier, String::from("a"));
              assert_eq!(argu_list.len(), 2);
          },
          _ => {}
        }
    }

    #[test]
    // 方法变量的申明
    fn test_fn_type_declare() {
        let binding = String::from("fn void (fn int ()) a = b");
        let pairs = CalculatorParser::parse(Rule::declare_stat, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse(p).unwrap();
        println!("{:#?}", res);

        match &res {
            AstNodeType::Declaration { declare_type, identifier, additive } => {
                assert_eq!(identifier.as_str(), String::from("a"));
            }
            _ => {}
        }
    }

    #[test]
    fn test_declare() {
        let binding = String::from("int b=a*2;");
        let pairs = CalculatorParser::parse(Rule::statement, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_stat(p.into_inner());
        // println!("{:#?}", res);
        assert_eq!(res.len(), 1);
        match &res[0] {
            AstNodeType::Declaration {
                declare_type,
                identifier,
                additive,
            } => {
                assert_eq!(identifier, "b")
            }
            _ => {}
        }
    }

    #[test]
    fn test_return() {
        let binding = String::from("return a;");
        let pairs = CalculatorParser::parse(Rule::equation, &binding).unwrap();
        let p = pairs;
        // println!("{:#?}", p);
        let rss = parse_simple(p);
        let res = rss.get(0).unwrap();
        // println!("{:#?}", res);
        match &res {
            AstNodeType::Statement { child } => {
                assert_eq!(child.len(), 1)
            }
            _ => {
                panic!("return 语句出错")
            }
        }
    }

    #[test]
    fn test_if() {
        let binding = String::from("if (8>4) {a;} else {b;}");
        let pairs = CalculatorParser::parse(Rule::if_statement, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_if(p);
        println!("{:#?}", res);

        match &res {
            AstNodeType::IfStatement {
                judge_stat,
                if_stat,
                else_stat,
            } => {
                assert_eq!(else_stat.is_none(), false)
            }
            _ => {}
        }
    }

    #[test]
    fn test_block() {
        let binding = String::from("{a;}");
        let pairs = CalculatorParser::parse(Rule::block, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_block(p);
        // println!("{:#?}", res);

        match &res {
            AstNodeType::Block { statements } => match statements {
                Some(s) => {
                    assert_eq!(s.len(), 1)
                }
                None => {
                    assert_eq!(1, 0)
                }
            },
            _ => {}
        }
    }

    #[test]
    fn test_judge() {
        let binding = String::from("8>5");
        let pairs = CalculatorParser::parse(Rule::judge_stat, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_judge(p);
        // println!("{:#?}", res);

        match &res {
            AstNodeType::JudgeExp { left, right, judge } => {
                assert_eq!(judge, ">")
            }
            _ => {}
        }
    }

    #[test]
    fn test_judge_state() {
        let binding = String::from("4>2;");
        let pairs = CalculatorParser::parse(Rule::statement, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_stat(p.into_inner());
        // println!("{:#?}", res);
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_stat() {
        let binding = String::from("b=a*2;");
        let pairs = CalculatorParser::parse(Rule::statement, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_stat(p.into_inner());
        // println!("{:#?}", res);
        assert_eq!(res.len(), 1);
        match &res[0] {
            AstNodeType::AssignmentStatement { ident, additive } => {
                assert_eq!(ident, "b")
            }
            _ => {}
        }
    }

    #[test]
    fn test_multi() {
        let binding = String::from("a*2");
        let pairs = CalculatorParser::parse(Rule::multi, &binding).unwrap();
        let p = pairs.peek().unwrap();
        let res = parse_mul(p);
        // println!("{:#?}", res);
        match res {
            AstNodeType::MulitiExp { child } => {
                assert_eq!(child.len(), 2)
            }
            _ => {
                //
            }
        }
    }

    #[test]
    fn test_add() {
        let binding = String::from("3+a*2");
        let pairs = CalculatorParser::parse(Rule::additive, &binding).unwrap();
        let p = pairs.peek().unwrap();
        // println!("{:#?}", p);

        let res = parse_add(p);
        // println!("{:#?}", res);
        match res {
            AstNodeType::AdditiveExp { child } => {
                assert_eq!(child.len(), 2)
            }
            _ => {
                //
            }
        }
    }
    #[test]
    fn test_main() {
        let res = parse_file();
        println!("{:#?}", res)
    }
}
