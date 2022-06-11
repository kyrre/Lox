use rlox::ast::{AstPrinter, Expr};
use rlox::tokens::{Literal, Token, TokenType};

fn test_ast() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token {
                token_type: TokenType::MINUS,
                lexeme: "-".to_string(),
                literal: Literal::None,
                line: 1,
            },
            right: Box::new(Expr::Literal {
                value: Literal::Number(123 as f64),
            }),
        }),
        operator: Token {
            token_type: TokenType::STAR,
            lexeme: "*".to_string(),
            literal: Literal::None,
            line: 1,
        },
        right: Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                value: Literal::Number(45.67),
            }),
        }),
    };

    let a = AstPrinter {};

    println!("{}", a.print(&expr));
}

fn main() {
    test_ast();
}
