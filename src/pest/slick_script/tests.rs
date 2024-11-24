use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main();
    }

    #[test]
    fn test_base() {
        let code = r#"int a = 3;
    a = 4;
    a;"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Number(n) => assert_eq!(n, 4),
            _ => {}
        }
    }

    #[test]
    fn test_judge() {
        let code = r#"3>4;"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Boolean(n) => assert_eq!(n, false),
            _ => {}
        }
    }

    #[test]
    fn test_if() {
        let code = r#"int a = 0;
        if (3<2) {
          a = 4;
        } else {
          a = 1;
        }
          a;"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Number(n) => assert_eq!(n, 1),
            _ => {}
        }
    }

    #[test]
    fn test_block() {
        let code = r#"int b = 0;
          if (3<2) {
            int b = 4;
          } else {
            int b = 1;
          }
            b;"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Number(n) => assert_eq!(n, 0),
            _ => {}
        }
    }

    #[test]
    fn test_fn_declare() {
        let code = r#"int a () { 
            int b = 4;
            return b;
          }
            a;"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::String(n) => assert_eq!(n, String::from("closure a")),
            _ => {}
        }
    }

    #[test]
    fn test_fn_assign() {
        let code = r#"int a () { 
          int b = 4;
          return b;
        }
      
        fn int () b  = a;
        
        b;"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::String(n) => assert_eq!(n, String::from("closure a")),
            _ => {}
        }
    }

    #[test]
    fn test_fn_call() {
        let code = r#"int a (int c) { 
          int b = c;
          return b;
        }
      
        a(1*2);"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Number(n) => assert_eq!(n, 2),
            _ => {}
        }
    }

    #[test]
    fn test_fn_call_no_args() {
        let code = r#"int a () { 
          int b = 3;
          return b;
        }
      
        a();"#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Number(n) => assert_eq!(n, 3),
            _ => {}
        }
    }

    #[test]
    fn test_closure_call() {
        let code = r#"int a () { 
          int c = 6;
          int b () {
            c = c + 1;
            return c;
          }
          return b;
        }
      
        fn int () c = a();
        c();
        "#;
        let res = test_entry(code.to_string());
        // println!("{:#?}", res);
        match res {
            ValType::Number(n) => assert_eq!(n, 7),
            _ => {}
        }
    }
}
