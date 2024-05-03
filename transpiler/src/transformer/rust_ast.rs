use std::fmt::Display;

use crate::ast::{self, Expr, Node, TLNode, Type};

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{n}"),
            Expr::FPNumber(fpn) => write!(f, "{fpn}"),
            Expr::Variable(v) => write!(f, "{v}"),
            Expr::Neg(stmt) => write!(f, "(-{stmt})"),
            Expr::BitNeg(stmt) => write!(f, "(!{stmt})"),
            Expr::Op { lhs, op, rhs } => write!(f, "({lhs} {op} {rhs})"),
            Expr::Call { name, args } => write!(f, "{name}{}", args.to_string()),
        }
    }
}

trait ToString {
    fn to_string(&self) -> String;
}

//https://stackoverflow.com/questions/37374077/how-to-implement-the-tostring-trait-to-create-a-comma-delimited-string-without-a
impl ToString for Vec<Variable> {
    fn to_string(&self) -> String {
        let mut ret_str = String::new();
        ret_str += "(";
        let mut str = "";
        for stmt in self {
            ret_str += str;
            ret_str += &stmt.to_string();
            str = ",";
        }
        ret_str += ")";

        ret_str
    }
}
impl ToString for Vec<Expr> {
    fn to_string(&self) -> String {
        let mut ret_str = String::new();
        ret_str += "(";
        let mut str = "";
        for stmt in self {
            ret_str += str;
            ret_str += &stmt.to_string();
            str = ",";
        }
        ret_str += ")";

        ret_str
    }
}
impl ToString for Vec<Stmt> {
    fn to_string(&self) -> String {
        let mut ret_str = String::new();
        for stmt in self {
            ret_str += &stmt.to_string();
            //ret_str += ";";
        }
        ret_str
    }
}

pub enum Stmt {
    If {
        condition: Expr,
        body: Vec<Stmt>,
        else_if: Vec<(Expr, Vec<Stmt>)>,
        or_else: Vec<Stmt>,
    },

    For {
        var_name: String,
        iterator: Expr,
        body: Vec<Stmt>,
    },

    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    Let {
        var: Variable,
        val: Expr,
    },

    SetVar {
        name: String,
        op: String,
        value: Expr,
    },

    Return(Option<Expr>),
    Expr(Expr),

    Continue,
    Break,
    // Used to presurve spacing and vibes
    NewLine,
    Comment(String),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expr(expr) => write!(f, "{expr};"),
            Stmt::If {
                condition,
                body,
                else_if,
                or_else,
            } => {
                let mut else_if_str = String::new();
                for (cond, body) in else_if {
                    else_if_str += &format!("else if {cond} {{\n {} }}\n", body.to_string());
                }

                write!(
                    f,
                    "if {condition} {{\n {} }}\n {else_if_str} else {{\n {} }}\n",
                    body.to_string(),
                    or_else.to_string()
                )
            }
            Stmt::For {
                var_name,
                iterator,
                body,
            } => write!(
                f,
                "for {var_name} in {iterator} {{\n {}}}\n",
                body.to_string()
            ),
            Stmt::While { condition, body } => {
                write!(f, "while {condition} {{ \n {} }}\n", body.to_string())
            }

            Stmt::Let { var, val } => write!(f, "let mut {var} = {val};\n"),
            Stmt::SetVar { name, op, value } => write!(f, "{name} {op} {value};\n"),

            Stmt::Return(body) => write!(
                f,
                "return {};\n",
                body.as_ref()
                    .map(|a| a.to_string())
                    .unwrap_or(String::new())
            ),
            Stmt::Continue => write!(f, "continue;\n"),
            Stmt::Break => write!(f, "break;\n"),

            Stmt::NewLine => write!(f, "\n"),
            Stmt::Comment(comment) => write!(f, "//{comment}\n"),
        }
    }
}

impl From<Node> for Stmt {
    fn from(value: Node) -> Self {
        match value {
            Node::If {
                condition,
                body,
                elif,
                or_else,
            } => Self::If {
                condition,
                body: body.into_iter().map(|n| n.into()).collect(),
                else_if: elif
                    .into_iter()
                    .map(|(expr, node)| (expr, node.into_iter().map(|n| n.into()).collect()))
                    .collect(),
                or_else: or_else.into_iter().map(|n| n.into()).collect(),
            },

            Node::For {
                var_name,
                iterator,
                body,
            } => {
                let iterator = match iterator {
                    ast::Iterator::Range { start, step, end } => {
                        if step == 1 {
                            Expr::Op {
                                lhs: Box::new(Expr::Number(start)),
                                op: "..".to_string(),
                                rhs: Box::new(Expr::Number(end)),
                            }
                        } else {
                            todo!()
                        }
                    }
                };

                Self::For {
                    var_name,
                    iterator,
                    body: body.into_iter().map(|n| n.into()).collect(),
                }
            }
            Node::While { condition, body } => Self::While {
                condition,
                body: body.into_iter().map(|n| n.into()).collect(),
            },
            Node::InitVar { var } => Self::Let {
                var: var.clone().into(),
                val: var
                    .default_value
                    .unwrap_or(Expr::Variable("None".into()))
                    .clone(),
            },
            Node::SetVar { name, op, value } => Self::SetVar { name, op, value },
            Node::Expr(e) => Self::Expr(e),
            Node::Return(body) => Self::Return(body),

            Node::Continue => Self::Continue,
            Node::Break => Self::Break,

            Node::NewLine => Self::NewLine,
            Node::Comment(c) => Self::Comment(c),
        }
    }
}

pub struct Variable {
    pub name: String,
    pub type_: Type,
}

impl From<ast::Variable> for Variable {
    fn from(value: ast::Variable) -> Self {
        //assert!(value.default_value.is_none(), "This isn't implemented");
        Self {
            name: value.name,
            type_: value.type_,
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.type_)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::None => write!(f, "Option<Gd<Variant>>"),
            Type::Auto => write!(f, "_"),
            Type::Some(type_) => {
                let type_ = match type_.as_str() {
                    "int" => "i64",
                    "float" => "f64",
                    _ => todo!(),
                };
                write!(f, "{type_}")
            }
        }
    }
}

enum Item {
    Function {
        name: String,
        args: Vec<Variable>,
        return_type: Type,
        body: Vec<Stmt>,
    },
}
impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Function {
                name,
                args,
                return_type,
                body,
            } => {
                write!(
                    f,
                    "fn {name}{} -> {return_type} {{\n {} }}\n",
                    args.to_string(),
                    body.to_string()
                )
            }
        }
    }
}

pub fn get_rust_src(ast: Vec<TLNode>) -> String {
    let mut class_name = "UnknownClassName".to_string(); // todo pass script name here
    let mut extends = "Object".to_string();

    let mut functions = String::new();

    for node in ast {
        match node {
            TLNode::ClassName(name) => class_name = name.clone(),
            TLNode::Extends(class) => extends = class.clone(),
            TLNode::Function {
                name,
                args,
                return_type,
                body,
            } => {
                let body = body.into_iter().map(|n| n.into()).collect();
                let args = args.into_iter().map(|v| v.into()).collect();
                let item = Item::Function {
                    name,
                    args,
                    return_type,
                    body,
                };

                functions += "#[func]\n";
                functions += &item.to_string();
            }
            TLNode::Comment(c) => functions += &format!("//{c}\n"),
            TLNode::NewLine => functions += "\n",
        }
    }

    format!(
        "

// Generated by GDScript-transpiler
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base={extends})]
struct {class_name} {{
// TODO vars
base: Base<{extends}>
}}

#[godot_api]
impl {class_name} {{
{functions}
}}


"
    )
}
