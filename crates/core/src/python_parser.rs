use crate::language_parser::{
    GenericFunctionDef, GenericTypeDef, Language, LanguageParser, TypeDefKind,
};
use crate::tree::TreeNode;
use std::error::Error;
use std::rc::Rc;
use tree_sitter::{Node, Parser};

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

        Ok(Self { parser })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn convert_node(&self, node: Node, source: &str, id_counter: &mut usize) -> TreeNode {
        let current_id = *id_counter;
        *id_counter += 1;

        let label = node.kind().to_string();
        let value = match node.kind() {
            "identifier" | "string" | "integer" | "float" | "true" | "false" | "none" => {
                node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
            }
            _ => "".to_string(),
        };

        let mut tree_node = TreeNode::new(label, value, current_id);

        for child in node.children(&mut node.walk()) {
            let child_node = self.convert_node(child, source, id_counter);
            tree_node.add_child(Rc::new(child_node));
        }

        tree_node
    }

    fn extract_functions_from_node(
        &self,
        node: Node,
        source: &str,
        class_name: Option<&str>,
    ) -> Vec<GenericFunctionDef> {
        let mut functions = Vec::new();

        // Visit all nodes
        fn visit_node(
            node: Node,
            source: &str,
            functions: &mut Vec<GenericFunctionDef>,
            class_name: Option<&str>,
        ) {
            match node.kind() {
                "function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            let params_node = node.child_by_field_name("parameters");
                            let body_node = node.child_by_field_name("body");

                            let params = extract_params(params_node, source);

                            functions.push(GenericFunctionDef {
                                name: name.to_string(),
                                start_line: node.start_position().row as u32 + 1,
                                end_line: node.end_position().row as u32 + 1,
                                body_start_line: body_node
                                    .map(|n| n.start_position().row as u32 + 1)
                                    .unwrap_or(0),
                                body_end_line: body_node
                                    .map(|n| n.end_position().row as u32 + 1)
                                    .unwrap_or(0),
                                parameters: params,
                                is_method: class_name.is_some(),
                                class_name: class_name.map(|s| s.to_string()),
                            });
                        }
                    }
                }
                "decorated_definition" => {
                    // Check if it decorates a function
                    if let Some(child) = node.child(node.child_count().saturating_sub(1)) {
                        if child.kind() == "function_definition" {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                    let params_node = child.child_by_field_name("parameters");
                                    let body_node = child.child_by_field_name("body");

                                    let params = extract_params(params_node, source);

                                    functions.push(GenericFunctionDef {
                                        name: name.to_string(),
                                        start_line: node.start_position().row as u32 + 1,
                                        end_line: node.end_position().row as u32 + 1,
                                        body_start_line: body_node
                                            .map(|n| n.start_position().row as u32 + 1)
                                            .unwrap_or(0),
                                        body_end_line: body_node
                                            .map(|n| n.end_position().row as u32 + 1)
                                            .unwrap_or(0),
                                        parameters: params,
                                        is_method: class_name.is_some(),
                                        class_name: class_name.map(|s| s.to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
                "class_definition" => {
                    // Don't recurse into nested classes when we're already in a class
                    if class_name.is_none() {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                                // Recursively extract methods from this class
                                let mut subcursor = node.walk();
                                for child in node.children(&mut subcursor) {
                                    visit_node(child, source, functions, Some(name));
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Continue traversing for other node types
                    let mut subcursor = node.walk();
                    for child in node.children(&mut subcursor) {
                        visit_node(child, source, functions, class_name);
                    }
                }
            }
        }

        fn extract_params(params_node: Option<Node>, source: &str) -> Vec<String> {
            if let Some(node) = params_node {
                let mut params = Vec::new();
                let mut cursor = node.walk();

                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "identifier" => {
                            if let Ok(param_text) = child.utf8_text(source.as_bytes()) {
                                params.push(param_text.to_string());
                            }
                        }
                        "typed_parameter" | "default_parameter" => {
                            if let Some(ident) = child.child_by_field_name("name") {
                                if let Ok(param_text) = ident.utf8_text(source.as_bytes()) {
                                    params.push(param_text.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }

                params
            } else {
                Vec::new()
            }
        }

        visit_node(node, source, &mut functions, class_name);
        functions
    }
}

impl LanguageParser for PythonParser {
    fn parse(&mut self, source: &str, _filename: &str) -> Result<Rc<TreeNode>, Box<dyn Error>> {
        let tree = self.parser.parse(source, None).ok_or("Failed to parse Python source")?;

        let root_node = tree.root_node();
        let mut id_counter = 0;
        Ok(Rc::new(self.convert_node(root_node, source, &mut id_counter)))
    }

    fn extract_functions(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Vec<GenericFunctionDef>, Box<dyn Error>> {
        let tree = self.parser.parse(source, None).ok_or("Failed to parse Python source")?;

        let root_node = tree.root_node();
        Ok(self.extract_functions_from_node(root_node, source, None))
    }

    fn extract_types(
        &mut self,
        source: &str,
        _filename: &str,
    ) -> Result<Vec<GenericTypeDef>, Box<dyn Error>> {
        let tree = self.parser.parse(source, None).ok_or("Failed to parse Python source")?;

        let root_node = tree.root_node();
        let mut types = Vec::new();

        fn visit_node_for_types(node: Node, source: &str, types: &mut Vec<GenericTypeDef>) {
            if node.kind() == "class_definition" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                        types.push(GenericTypeDef {
                            name: name.to_string(),
                            kind: TypeDefKind::Class,
                            start_line: node.start_position().row as u32 + 1,
                            end_line: node.end_position().row as u32 + 1,
                        });
                    }
                }
            }

            // Continue traversing
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node_for_types(child, source, types);
            }
        }

        visit_node_for_types(root_node, source, &mut types);
        Ok(types)
    }

    fn language(&self) -> Language {
        Language::Python
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_functions() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
def hello(name):
    return f"Hello, {name}!"

def add(a, b=0):
    return a + b

class Calculator:
    def __init__(self):
        self.result = 0
    
    def add(self, x):
        self.result += x
        return self.result
"#;

        let functions = parser.extract_functions(source, "test.py").unwrap();
        assert_eq!(functions.len(), 4);
        assert_eq!(functions[0].name, "hello");
        assert_eq!(functions[1].name, "add");
        assert!(!functions[1].is_method);
        assert_eq!(functions[2].name, "__init__");
        assert!(functions[2].is_method);
        assert_eq!(functions[2].class_name, Some("Calculator".to_string()));
        assert_eq!(functions[3].name, "add");
        assert!(functions[3].is_method);
    }

    #[test]
    fn test_python_classes() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
class User:
    def __init__(self, name):
        self.name = name

class Admin(User):
    def __init__(self, name, level):
        super().__init__(name)
        self.level = level
"#;

        let types = parser.extract_types(source, "test.py").unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].name, "User");
        assert_eq!(types[0].kind, TypeDefKind::Class);
        assert_eq!(types[1].name, "Admin");
    }
}
