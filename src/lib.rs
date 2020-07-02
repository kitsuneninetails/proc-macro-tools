use lazy_static::lazy_static;
use proc_macro;
use regex::{Captures, Regex};

lazy_static! {
    static ref FN_PATTERN: Regex =
        Regex::new(r#"^([\w\W]*?) *(pub +)?(async +)?fn +([\w\W]*?)(\([\w\W]*?) +?(->[\w\W]*?)?[ \n]*\{([\S\s]*)\}"#)
            .unwrap();
}

fn add_space_or_empty(input: &str) -> String {
    if !input.is_empty() {
        format!("{} ", input)
    } else {
        format!("")
    }
}

pub struct FunctionDecl {
    pub func_prologue: String,
    pub pub_str: String,
    pub async_str: String,
    pub fn_name: String,
    pub fn_decl: String,
    pub ret_decl: String,
    pub fn_body: String,
}

impl FunctionDecl {
    pub fn from_string(in_str: String) -> Self {
        let caps: Captures = FN_PATTERN
            .captures(in_str.as_ref())
            .unwrap_or_else(|| panic!("Can only use on a function declaration"));

        if caps.len() != 8 {
            panic!("Must be a proper fn declaration");
        }

        let func_prologue = caps[1].trim_matches(' ').to_string();
        let pub_str = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim().to_string();
        let async_str = caps.get(3).map(|m| m.as_str()).unwrap_or("").trim().to_string();
        let fn_name = caps[4].trim().to_string();
        let fn_decl = caps[5].trim().to_string();

        let ret_decl = caps
            .get(6)
            .map(|m| {
                let mut s = m.as_str().to_string();
                let (p, _) = s.char_indices().nth(2).unwrap();
                s.drain(0..p);
                s.trim().to_string()
            })
            .unwrap_or("".to_string());

        let fn_body = caps[7].trim().to_string();

        FunctionDecl {
            func_prologue,
            pub_str,
            async_str,
            fn_name,
            fn_decl,
            ret_decl,
            fn_body,
        }
    }

    pub fn func_prelude(&self) -> String {
        format!(
            "{}{}{}fn {}{}{} {{",
            self.func_prologue,
            add_space_or_empty(&self.pub_str),
            add_space_or_empty(&self.async_str),
            self.fn_name,
            self.fn_decl,
            if !self.ret_decl.is_empty() {
                format!(" -> {}", self.ret_decl)
            } else {
                "".to_string()
            }
        )
    }

    pub fn func_end(&self) -> String {
        format!("}}")
    }

    pub fn into_func_body(self, body_add: String) -> String {
        format!("{}\n{}\n{}", self.func_prelude(), body_add, self.func_end())
    }
}

#[cfg(test)]
mod tests {
    use crate::FunctionDecl;

    #[test]
    fn test_func_simple_one_line() {
        let test = "fn simple_sameline() {}".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "simple_sameline");
        assert_eq!(funcdecl.fn_decl, "()");
        assert_eq!(funcdecl.fn_body, "");
    }

    #[test]
    fn test_func_simple_new_line() {
        let test = "fn simple_newline() {
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "simple_newline");
        assert_eq!(funcdecl.fn_decl, "()");
        assert_eq!(funcdecl.ret_decl, "");
        assert_eq!(funcdecl.fn_body, "");
    }

    #[test]
    fn test_func_simple_new_line_brace_new_line() {
        let test = "fn simple_newline_brace()
        {
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "simple_newline_brace");
        assert_eq!(funcdecl.fn_decl, "()");
        assert_eq!(funcdecl.ret_decl, "");
        assert_eq!(funcdecl.fn_body, "");
    }

    #[test]
    fn test_func_simple_with_params() {
        let test = "fn with_params(_: String) {}".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "with_params");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "");
        assert_eq!(funcdecl.fn_body, "");
    }

    #[test]
    fn test_func_body_no_return() {
        let test = "fn with_body(_: String) {
            let _ = \"\".to_string();
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "with_body");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "");
        assert_eq!(funcdecl.fn_body, "let _ = \"\".to_string();");
    }

    #[test]
    fn test_func_body_with_return() {
        let test = "fn with_return(_: String) -> String {
            \"\".to_string()
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "with_return");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "String");
        assert_eq!(funcdecl.fn_body, "\"\".to_string()");
    }

    #[test]
    fn test_func_pub_with_return() {
        let test = "pub fn with_return(_: String) -> String {
            \"\".to_string()
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "pub");
        assert_eq!(funcdecl.async_str, "");
        assert_eq!(funcdecl.fn_name, "with_return");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "String");
        assert_eq!(funcdecl.fn_body, "\"\".to_string()");
    }

    #[test]
    fn test_func_async_with_return() {
        let test = "async fn with_return(_: String) -> String {
            \"\".to_string()
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "");
        assert_eq!(funcdecl.async_str, "async");
        assert_eq!(funcdecl.fn_name, "with_return");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "String");
        assert_eq!(funcdecl.fn_body, "\"\".to_string()");
    }

    #[test]
    fn test_func_async_pub_with_return() {
        let test = "pub async fn with_return(_: String) -> String {
            \"\".to_string()
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "");
        assert_eq!(funcdecl.pub_str, "pub");
        assert_eq!(funcdecl.async_str, "async");
        assert_eq!(funcdecl.fn_name, "with_return");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "String");
        assert_eq!(funcdecl.fn_body, "\"\".to_string()");
    }

    #[test]
    fn test_func_async_pub_prelude_with_return() {
        let test = "#[some_macro]
        pub async fn with_return(_: String) -> String {
            \"\".to_string()
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "#[some_macro]\n");
        assert_eq!(funcdecl.pub_str, "pub");
        assert_eq!(funcdecl.async_str, "async");
        assert_eq!(funcdecl.fn_name, "with_return");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "String");
        assert_eq!(funcdecl.fn_body, "\"\".to_string()");
    }

    #[test]
    fn test_func_async_pub_large_body() {
        let test = "#[some_macro]
        pub async fn with_return(_: String) -> String {
            let foo = \"\".to_string();
            let bar = foo.trim();
            bar
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        assert_eq!(funcdecl.func_prologue, "#[some_macro]\n");
        assert_eq!(funcdecl.pub_str, "pub");
        assert_eq!(funcdecl.async_str, "async");
        assert_eq!(funcdecl.fn_name, "with_return");
        assert_eq!(funcdecl.fn_decl, "(_: String)");
        assert_eq!(funcdecl.ret_decl, "String");
        assert_eq!(funcdecl.fn_body, "let foo = \"\".to_string();
            let bar = foo.trim();
            bar");
    }

    #[test]
    fn test_func_async_pub_large_body_into_body() {
        let test = "#[some_macro]
        pub async fn with_return(_: String) -> String {
            let foo = \"\".to_string();
            let bar = foo.trim();
            bar
        }".to_string();
        let funcdecl = FunctionDecl::from_string(test);
        let body = funcdecl.fn_body.clone();
        let expected = "#[some_macro]\npub async fn with_return(_: String) -> String \
            {\nlet foo = \"\".to_string();\n            let bar = foo.trim();\n            bar\n}";
        assert_eq!(funcdecl.into_func_body(body), expected);
    }
}

