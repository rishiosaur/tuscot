use crate::token::Token;
use crate::{
    ast::{Expression, Program, Statement},
    errors::error,
    objects::{Context, Object},
    token::TokenType,
};

fn join_to_string<'a>(toks: Vec<Token<'a>>) -> &str {
    toks.iter()
        .fold("".to_string(), |mut i, j| {
            if let TokenType::Ident(id) = j.kind {
                i.push_str(id);
                i
            } else {
                error(format!("Statement not supported."));
                panic!();
            }
        })
        .as_str()
}

pub fn eval<'a>(program: Program<'a>) {
    let mut context = Context::new(None);
    let mut result = &Object::Null;

    for statement in program.statements {
        result = eval_statement(&statement, &mut context);
    }

    // result
}

pub fn eval_statement<'a>(
    statement: &'a Statement<'a>,
    context: &'a mut Context<'a>,
) -> &'a Object<'a> {
    match *statement {
        Statement::AssignStatement {
            token,
            expression,
            defined,
        } => {
            let evaluated_expr = eval_expr(expression, context);
            if let Expression::DefinitionIdentifier { idents } = *defined {
                if idents.len() > 1 {
                    if let Object::Array(elems) = evaluated_expr {
                        for (index, ident) in idents.iter().enumerate() {
                            let TokenType::Ident(z) = ident.kind;
                            context.set(z, elems[index]);
                        }
                        return &Object::Null;
                    } else {
                        error(format!("{} Destructuring assignment is only valid for expressions that return an array.", token));
                        panic!();
                    }
                }
                if let TokenType::Ident(z) = idents[0].kind {
                    context.set(z, evaluated_expr);
                } else {
                    error(format!("Statement not supported."));
                    panic!();
                }

                return &Object::Null;
            }
            error(format!(
                "{:?} Definition statement isn't a definition????",
                statement
            ));
            panic!();
        }
        Statement::BlockStatement { token, statements } => {
            let eval = evaluate_block_statement(statement, context);
            return eval;
        }
        Statement::ExpressionStatement { token, expression } => {
            return eval_expr(expression, context);
        }
        Statement::UpdateStatement {
            token,
            ident,
            expression,
        } => {
            let evaluated_expr = eval_expr(expression, context);
            match *ident {
                Expression::NormalIdentifier { idents } => {
                    return context.update(join_to_string(idents), evaluated_expr);
                }
                Expression::PrefixExpression { token, right } => {
                    if token.kind == TokenType::Asterisk {
                        if let Expression::NormalIdentifier { idents } = *right {
                            let reference =
                                context.get(join_to_string(idents)).unwrap_or_else(|| {
                                    error(format!("{} Undefined reference.", idents[0].position));
                                    panic!();
                                });
                            if let Object::Reference { to, .. } = reference {
                                let Expression::NormalIdentifier { idents } = **to;
                                return context.update(join_to_string(idents), evaluated_expr);
                            } else {
                                error(format!("{} Not a reference.", idents[0].position));
                                panic!();
                            }
                        } else {
                            error(format!(" Statement not supported.",));
                            panic!();
                        }
                    } else {
                        error(format!("{:?} Statement not supported.", statement));
                        panic!();
                    }
                }
                _ => {
                    error(format!("{:?} Statement not supported.", statement));
                    panic!();
                }
            }
        }
        Statement::ReturnStatement { value, .. } => {
            return &Object::ReturnValue {
                value: eval_expr(value, context),
            };
        }
        _ => {
            error(format!("{:?} Statement not supported.", statement));
            panic!();
        }
    }
}

pub fn eval_expr<'a>(
    expression: Box<Expression<'a>>,
    context: &'a mut Context<'a>,
) -> &'a Object<'a> {
    match *expression {
        Expression::NormalIdentifier { idents } => {
            if let Some(val) = context.get(join_to_string(idents)) {
                return val;
            }

            error(format!(
                "{} Could not find identifier '{}' in current lexical context.",
                idents[0],
                join_to_string(idents)
            ));
            panic!();
        }
        Expression::StringLiteral { token } => match token.kind {
            TokenType::String(string) => &Object::String(string.to_owned()),
            _ => {
                error(format!(
                    "{} Expected string at string literal.",
                    token.position
                ));
                panic!();
            }
        },
        Expression::ArrayLiteral { token, elements } => {
            let elements = eval_expressions(elements, context);
            return &Object::Array(elements);
        }
        Expression::BooleanLiteral { token } => {
            let TokenType::Boolean(b) = token.kind;
            return &Object::Boolean(b);
        }
        Expression::UnderscoreLiteral { token } => &Object::Underscore,
        Expression::IntegerLiteral { token } => {
            let TokenType::Integer(i) = token.kind;
            return &Object::Integer(i as isize);
        }
        Expression::FloatLiteral { token } => {
            let TokenType::Float(i) = token.kind;
            return &Object::Float(i as f64);
        }
        Expression::FunctionLiteral {
            token,
            parameters,
            statements,
        } => {
            let Expression::DefinitionIdentifier { idents } = *parameters;
            return &Object::Function {
                parameters: idents,
                body: statements,
                context,
            };
        }
        Expression::CallExpression {
            token,
            parameters,
            function,
        } => {
            if let Expression::NormalIdentifier { idents } = *function {
                // let ident = idents.join(".");
                let func = context.get(join_to_string(idents)).unwrap();

                if let Object::Function { .. } = func {
                    let args = eval_expressions(parameters, context);
                    return apply_function(func, args);
                }

                error(format!(
                    "{} '{:?}' is not a function.",
                    idents[0].position, func
                ));
                panic!();
            } else {
                error(format!(
                    "{} '{:?}' is not a function.",
                    token.position, function
                ));
                panic!();
            }
        }
        Expression::PrefixExpression { token, right } => {
            let expr = eval_expr(right, context);
            match token.kind {
                TokenType::Ampersand => &Object::Reference {
                    to: right,
                    value: expr,
                },
                TokenType::Asterisk => {
                    if let Expression::NormalIdentifier { idents } = *right {
                        let Object::Reference { value, .. } =
                            context.get(join_to_string(idents)).unwrap();
                        return *value;
                    } else {
                        error(format!(
                            "{} Can only dereference an identifier.",
                            token.position
                        ));
                        panic!();
                    }
                }
                TokenType::Bang => match expr {
                    Object::Boolean(b) => &Object::Boolean(!*b),
                    _ => &Object::Boolean(false),
                },
                TokenType::Minus => {
                    match expr {
                        Object::Integer(i) => &Object::Integer(-*i),
                        _ => {
                            error(format!("{} Expected integer or float literal to come after negative operator.", token.position));
                            panic!();
                        }
                    }
                }
                _ => {
                    error(format!(
                        "{} Unknown prefix operator: '{:?}'.",
                        token.position, token.kind
                    ));
                    panic!()
                }
            }
        }
        // Expression::InfixExpression { token, right, left } => {
        //     let left = eval_expr(&left, context);
        //     let right = eval_expr(&right, context);
        //
        //
        //
        //
        // }

        // Expression::MatchExpression {
        //     token,
        //     default,
        //     pairs,
        // } => {}
        _ => {
            error(format!("{:?} Could not evaluate object.", expression));
            panic!();
        }
    }
}

// pub fn eval_prefix_expr()

fn eval_expressions<'a>(
    expressions: Vec<Box<Expression<'a>>>,
    context: &'a mut Context<'a>,
) -> Vec<&'a Object<'a>> {
    let mut res: Vec<&Object<'a>> = Vec::new();

    for expr in expressions {
        res.push(eval_expr(Box::new(*expr), context));
    }

    res
}

fn apply_function<'a>(func: &'a Object<'a>, args: Vec<&'a Object<'a>>) -> &'a Object<'a> {
    match func {
        Object::Function {
            parameters,
            body,
            context,
        } => {
            let mut extended = extend_local_ctx(func, args);
            let evaluated = evaluate_block_statement(body, &mut extended);
            return unwrap_return_value(evaluated);
        }
        _ => {
            error(format!("{:?} Function was not a function.", func));
            panic!();
        }
    }
}

fn extend_local_ctx<'a>(func: &'a Object<'a>, args: Vec<&'a Object<'a>>) -> Context<'a> {
    if let Object::Function {
        parameters,
        body,
        context,
    } = func
    {
        let mut ctx = Context::new(Some(*context));
        for (index, arg) in parameters.iter().enumerate() {
            if let TokenType::Ident(ident) = arg.kind {
                ctx.set(ident, args[index]);
            }
        }

        return ctx;
    }
    error(format!("{:?} Function was not a function.", func));
    panic!();
}

fn evaluate_block_statement<'a>(
    block: &'a Statement<'a>,
    context: &'a mut Context<'a>,
) -> &'a Object<'a> {
    if let Statement::BlockStatement { statements, .. } = block {
        for stmt in statements {
            let res = eval_statement(stmt, context);
            match res {
                Object::ReturnValue { .. } => return &res,
                _ => {}
            }
        }

        return &Object::Null;
    }

    error(format!("{:?} Block expr.", block));
    panic!();
}

fn unwrap_return_value<'a>(block: &Object<'a>) -> &'a Object<'a> {
    if let Object::ReturnValue { value } = block {
        return *value;
    }
    error(format!("{:?} Block expr.", block));
    panic!();
}
