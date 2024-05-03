#[derive(Debug, Clone)]
pub enum Expr {
    Number(u64),
    FPNumber(f64),
    Variable(String),

    Neg(Box<Expr>),
    BitNeg(Box<Expr>),
    Op {
        lhs: Box<Expr>,
        //op_code: OpCode,
        op: String, // This is not a good idea
        rhs: Box<Expr>,
    },

    Call {
        name: String,
        args: Vec<Expr>,
    },
    // Subscription x[index]
    // Attribute reference thing.x
    // if
    // Lambda

    // A lot more
}
/*
#[derive(Debug, Clone)]
pub enum OpCode {
    Mul,
    Div,
    Add,
    Sub,

    Eq,
    NotEq,
}*/

#[derive(Debug, Clone)]
pub enum Iterator {
    Range { start: u64, step: u64, end: u64 },
    /*Array {
        // TODO
    }*/
}
/*
impl Iterator {
    pub fn start_val(&self, var_name: &str) -> Vec<Node> {
        let value =
        match self {
            Iterator::Range { start, step: _, end: _ } => Expr::Number(*start)
        };

        vec![
            Node::InitVar { var: Variable { name: var_name.into(), type_: Type::Auto, default_value: Some(value) } }
        ]
    }

    pub fn condition(&self, var_name: &str) -> Expr {
        let value =
        match self {
            Iterator::Range { start: _, step: _, end } => end
        };

        Expr::Op { lhs: Box::new(Expr::Variable(var_name.into())), op: ">=".into(), rhs: Box::new(Expr::Number(*value)) }
    }

    pub fn step(&self, var_name: &str) -> Vec<Node> {
        let value =
        match self {
            Iterator::Range { start: _, step, end: _ } => step
        };

        vec![Node::SetVar { name: var_name.into(), op: "+=".into(), value: Expr::Number(*value) }]
    }
}
*/
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    // 1st is if to auto detect type, 2nd is type
    pub type_: Type,
    pub default_value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    None,
    Auto,
    Some(String),
}

impl From<Option<Option<String>>> for Type {
    fn from(value: Option<Option<String>>) -> Self {
        match value {
            Some(Some(t)) => Type::Some(t),
            Some(None) => Type::Auto,
            None => Type::None,
        }
    }
}
impl From<Option<String>> for Type {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(t) => Type::Some(t),
            None => Type::None,
        }
    }
}
/// Top level node
#[derive(Debug, Clone)]
pub enum TLNode {
    Function {
        name: String,
        args: Vec<Variable>,
        return_type: Type,
        body: Vec<Node>,
    },

    // Not implemented in compiler
    ClassName(String),
    // Not implemented in compiler
    Extends(String),

    // Not implemented in parser and compiler
    /*GlobalVar {
        var: Variable,
    },

    Const {
        name: String,
        value: Expr,
    },

    Enums {
        name: Option<String>,
        // TODO
    },

    Class {
        name: String,
        body: Vec<TLNode>,
    },

    Annotation {
        body: Box<TLNode>,
    }*/
    // signal
    // static
    // preload?
    // await yield

    // Used to presurve spacing and vibes
    NewLine,
    Comment(String),
}

#[derive(Debug, Clone)]
pub enum Node {
    If {
        condition: Expr,
        body: Vec<Node>,
        elif: Vec<(Expr, Vec<Node>)>,
        or_else: Vec<Node>,
    },

    // Not implemented in compiler
    For {
        var_name: String,
        iterator: Iterator,
        body: Vec<Node>,
    },

    // Not implemented in compiler
    While {
        condition: Expr,
        body: Vec<Node>,
    },

    InitVar {
        var: Variable,
    },

    SetVar {
        name: String,
        op: String,
        value: Expr,
    },

    /*Match {
        // TODO
    },

    Const {
        name: String,
        value: Expr
    },*/
    Expr(Expr),
    Return(Option<Expr>),

    Continue,
    Break,

    // Used to presurve spacing and vibes
    NewLine,
    Comment(String),
}
