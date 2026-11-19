#[derive(Debug)]
pub enum Program<'src> {
    Commands(Vec<Command<'src>>),
}

/// An arithmetic expression.
///
/// This may be an integer, a variable name, or an arithmetic operation with two more arithmetic expressions.
#[derive(Debug, Clone)]
pub enum AExpr<'src> {
    Int(u64),
    Variable(&'src str),
    Add(Box<AExpr<'src>>, Box<AExpr<'src>>),
    Subtract(Box<AExpr<'src>>, Box<AExpr<'src>>),
    Multiply(Box<AExpr<'src>>, Box<AExpr<'src>>),
}

/// A boolean expression
#[derive(Debug, Clone)]
pub enum BExpr<'src> {
    /// The `true` value
    True,
    /// The `false` value
    False,
    /// Boolean not: `not x`
    Not(Box<BExpr<'src>>),
    /// Boolean and: `x and y`
    And(Box<BExpr<'src>>, Box<BExpr<'src>>),
    /// Boolean or: `x or y`
    Or(Box<BExpr<'src>>, Box<BExpr<'src>>),
    /// Greater than: `x > y`
    Gt(Box<AExpr<'src>>, Box<AExpr<'src>>),
    /// Less than: `x < y`
    Lt(Box<AExpr<'src>>, Box<AExpr<'src>>),
    /// Equal to: `x == y`
    Eq(Box<AExpr<'src>>, Box<AExpr<'src>>),
    /// Not equal: `x <> y`
    Ne(Box<AExpr<'src>>, Box<AExpr<'src>>),
}

/// A command (statement).
///
/// This may be `skip`, an assignment, or a control flow statement (if, while).
/// `if` and `while` contain a list of more commands that shall be run.
#[derive(Debug, Clone)]
pub enum Command<'src> {
    Skip(),
    Assignment {
        name: &'src str,
        value: Box<AExpr<'src>>,
    },
    If {
        condition: Box<BExpr<'src>>,
        then_cmds: Vec<Command<'src>>,
        else_cmds: Vec<Command<'src>>,
    },
    While {
        condition: Box<BExpr<'src>>,
        cmds: Vec<Command<'src>>,
    },
}
