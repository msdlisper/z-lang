use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_scope() {
        let global_scope: Rc<RefCell<Scope>> = Rc::new(RefCell::new(Scope::Block(ScopeStruct {
            play_object: HashMap::new(),
            parent_scope: None,
        })));
        let mut first_frame = Frame {
            parent_frame: None,
            scope: global_scope,
        };
        first_frame.set(String::from("a"), Rc::new(ValType::Number(1)), true);

        // 创建新作用域
        let b_scope = Rc::new(RefCell::new(Scope::Block(ScopeStruct {
            play_object: HashMap::new(),
            parent_scope: None,
        })));
        first_frame.create_scope(b_scope);

        // 测试包含
        let has = first_frame.contains_key(&String::from("a"));
        assert_eq!(has, true);

        // 测试获取值
        let get_res = first_frame.get(&String::from("a")).unwrap();
        match *get_res {
            ValType::Number(n) => {
                assert_eq!(n, 1)
            },
            _ => {
                panic!("层级出问题了")
            }
        }


        // 测试获取值
        first_frame.set(String::from("a"), Rc::new(ValType::Number(0)), false);
        let get_res0 = first_frame.get(&String::from("a")).unwrap();
        match *get_res0 {
            ValType::Number(n) => {
                assert_eq!(n, 0)
            },
            _ => {
                panic!("层级出问题了")
            }
        }
    }
}
