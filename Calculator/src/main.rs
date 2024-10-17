use rocket::serde::json::Json;
use rocket::response::content::RawHtml;
use serde::Serialize;
use std::fs;
use std::str::FromStr;
use rocket::get;
use rocket::routes;
use rocket::launch;

// Структура для відповіді з результатом
#[derive(Serialize)]
struct CalcResult {
    result: Option<f64>,   // Для результату обчислення
    error: Option<String>, // Для помилки
}

// Відображення головної сторінки
#[get("/")]
fn index() -> Option<RawHtml<String>> {
    let html_content = fs::read_to_string("static/index.html").ok()?;
    Some(RawHtml(html_content))
}

// Функція обробки запиту з виразом та повернення результату
#[get("/calculate?<expression>")]
fn calculate(expression: String) -> Json<CalcResult> {
    match eval_expression(&expression) {
        Ok(result) => {
            if result.is_infinite() || result.is_nan() {
                Json(CalcResult {
                    result: None,
                    error: Some("Ділення на 0 неможливе".to_string()),
                })
            } else {
                Json(CalcResult {
                    result: Some(result),
                    error: None,
                })
            }
        }
        Err(e) => Json(CalcResult {
            result: None,
            error: Some(e),
        }),
    }
}

// Розбір виразу та виконання операції
fn eval_expression(expression: &str) -> Result<f64, String> {
    let tokens = tokenize_expression(expression)?;

    let postfix = infix_to_postfix(tokens)?;

    evaluate_postfix(postfix)
}

// Tokenize the input expression into numbers and operators
fn tokenize_expression(expression: &str) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = Vec::new(); // Changed from Vec<str> to Vec<String>
    let mut num_buffer = String::new();
    let mut chars = expression.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_digit(10) || c == '.' {
            num_buffer.push(c);
            chars.next();
        } else {
            if !num_buffer.is_empty() {
                tokens.push(num_buffer.clone());
                num_buffer.clear();
            }

            if c == '-' && (tokens.is_empty() || "+-*/(".contains(tokens.last().unwrap().as_str())) {
                num_buffer.push(c);
                chars.next();
                continue;
            }

            if "+-*/()".contains(c) {
                tokens.push(c.to_string());
                chars.next();
            } else if c == ' ' {
                chars.next();
            } else {
                return Err(format!("Unexpected character '{}'", c));
            }
        }
    }

    if !num_buffer.is_empty() {
        tokens.push(num_buffer);
    }

    Ok(tokens)
}

// Convert infix expression to postfix (Reverse Polish Notation)
fn infix_to_postfix(tokens: Vec<String>) -> Result<Vec<String>, String> {
    let mut output: Vec<String> = Vec::new();
    let mut operator_stack: Vec<String> = Vec::new(); // Changed from Vec<str> to Vec<String>

    for token in tokens {
        if let Ok(_) = f64::from_str(&token) {
            output.push(token);
        } else if "+-*/".contains(&token[..]) {
            while let Some(top_op) = operator_stack.last() {
                if precedence(top_op) >= precedence(&token) {
                    output.push(operator_stack.pop().unwrap());
                } else {
                    break;
                }
            }
            operator_stack.push(token);
        } else if token == "(" {
            operator_stack.push(token);
        } else if token == ")" {
            while let Some(top_op) = operator_stack.pop() {
                if top_op == "(" {
                    break;
                } else {
                    output.push(top_op);
                }
            }
        }
    }

    while let Some(op) = operator_stack.pop() {
        if op == "(" || op == ")" {
            return Err("Mismatched parentheses".to_string());
        }
        output.push(op);
    }

    Ok(output)
}

// Evaluate a postfix expression
fn evaluate_postfix(postfix: Vec<String>) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new(); // Stack now correctly holds f64 values

    for token in postfix {
        if let Ok(num) = f64::from_str(&token) {
            stack.push(num);
        } else if "+-*/".contains(&token[..]) {
            let b = stack.pop().ok_or("Insufficient values in expression")?;
            let a = stack.pop().ok_or("Insufficient values in expression")?;

            let result = match &token[..] {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => {
                    if b == 0.0 {
                        return Err("Неможна ділити на 0".to_string());
                    }
                    a / b
                }
                _ => unreachable!(),
            };
            stack.push(result);
        } else {
            return Err(format!("Unexpected token '{}'", token));
        }
    }

    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else {
        Err("The expression could not be evaluated".to_string())
    }
}

// Helper function to determine operator precedence
fn precedence(op: &str) -> i32 {
    match op {
        "+" | "-" => 1,
        "*" | "/" => 2,
        _ => 0,
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, calculate])
        .mount("/static", rocket::fs::FileServer::from("static")) // To serve static files like index.html
}
