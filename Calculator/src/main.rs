use rocket::serde::json::Json;
use rocket::response::content::RawHtml;
use serde::Serialize;
use std::fs;
use std::str::FromStr;
use rocket::get;
use rocket::routes;
use rocket::launch;

// Структура для відповіді з результатом обчислення
#[derive(Serialize)]
struct CalcResult {
    result: Option<f64>,   // Поле для результату обчислення (може бути None у випадку помилки)
    error: Option<String>, // Поле для повідомлення про помилку (може бути None, якщо помилки немає)
}

// Відображення головної сторінки. Читаємо HTML-файл та повертаємо його вміст як RawHtml
#[get("/")]
fn index() -> Option<RawHtml<String>> {
    let html_content = fs::read_to_string("static/index.html").ok()?; // Читаємо файл, якщо помилка - повертаємо None
    Some(RawHtml(html_content)) // Повертаємо вміст HTML-файлу як RawHtml
}

// Функція обробки запиту з виразом. Повертає JSON із результатом або повідомленням про помилку
#[get("/calculate?<expression>")]
fn calculate(expression: String) -> Json<CalcResult> {
    // Використовуємо eval_expression для обчислення введеного виразу
    match eval_expression(&expression) {
        // Якщо обчислення успішне
        Ok(result) => {
            // Перевіряємо на випадки ділення на 0 або некоректних обчислень (NaN)
            if result.is_infinite() || result.is_nan() {
                Json(CalcResult {
                    result: None,
                    error: Some("Ділення на 0 неможливе".to_string()), // Повертаємо повідомлення про помилку
                })
            } else {
                Json(CalcResult {
                    result: Some(result), // Повертаємо результат
                    error: None,
                })
            }
        }
        // Якщо була помилка в обчисленні
        Err(e) => Json(CalcResult {
            result: None, // Повертаємо None як результат
            error: Some(e), // Повертаємо текст помилки
        }),
    }
}

// Функція для розбору виразу і обчислення
fn eval_expression(expression: &str) -> Result<f64, String> {
    // Розбиваємо вираз на токени (числа та оператори)
    let tokens = tokenize_expression(expression)?;

    // Перетворюємо інфіксну нотацію на постфіксну (обратну польську нотацію)
    let postfix = infix_to_postfix(tokens)?;

    // Виконуємо обчислення постфіксного виразу
    evaluate_postfix(postfix)
}

// Функція для розбиття виразу на числа та оператори
fn tokenize_expression(expression: &str) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = Vec::new(); // Змінна для зберігання токенів
    let mut num_buffer = String::new(); // Буфер для чисел
    let mut chars = expression.chars().peekable(); // Використовуємо peekable для зручної роботи з символами

    while let Some(&c) = chars.peek() {
        // Якщо символ - цифра або точка, додаємо його до буфера чисел
        if c.is_digit(10) || c == '.' {
            num_buffer.push(c);
            chars.next();
        } else {
            // Якщо в буфері є число, додаємо його до токенів
            if !num_buffer.is_empty() {
                tokens.push(num_buffer.clone());
                num_buffer.clear();
            }

            // Обробка унарного мінусу
            if c == '-' && (tokens.is_empty() || "+-*/(".contains(tokens.last().unwrap().as_str())) {
                num_buffer.push(c);
                chars.next();
                continue;
            }

            // Якщо символ - оператор або дужки, додаємо його як окремий токен
            if "+-*/()".contains(c) {
                tokens.push(c.to_string());
                chars.next();
            } else if c == ' ' {
                chars.next(); // Ігноруємо пробіли
            } else {
                return Err(format!("Непередбачений символ '{}'", c)); // Помилка, якщо зустрічається невідомий символ
            }
        }
    }

    // Якщо в буфері залишилось число, додаємо його до токенів
    if !num_buffer.is_empty() {
        tokens.push(num_buffer);
    }

    Ok(tokens)
}

// Перетворення інфіксного виразу в постфіксний (обратну польську нотацію)
fn infix_to_postfix(tokens: Vec<String>) -> Result<Vec<String>, String> {
    let mut output: Vec<String> = Vec::new(); // Вихідний список для постфіксного виразу
    let mut operator_stack: Vec<String> = Vec::new(); // Стек для операторів

    for token in tokens {
        // Якщо токен - число, додаємо його до виходу
        if let Ok(_) = f64::from_str(&token) {
            output.push(token);
        } else if "+-*/".contains(&token[..]) {
            // Якщо токен - оператор, виконуємо обробку за пріоритетом
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
            // Обробка правої дужки
            while let Some(top_op) = operator_stack.pop() {
                if top_op == "(" {
                    break;
                } else {
                    output.push(top_op);
                }
            }
        }
    }

    // Додаємо всі залишки операторів у вихідний список
    while let Some(op) = operator_stack.pop() {
        if op == "(" || op == ")" {
            return Err("Невідповідність дужок".to_string()); // Помилка, якщо залишились незакриті дужки
        }
        output.push(op);
    }

    Ok(output)
}

// Функція для обчислення постфіксного виразу
fn evaluate_postfix(postfix: Vec<String>) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new(); // Стек для зберігання чисел

    for token in postfix {
        // Якщо токен - число, додаємо його до стека
        if let Ok(num) = f64::from_str(&token) {
            stack.push(num);
        } else if "+-*/".contains(&token[..]) {
            // Якщо токен - оператор, беремо два числа зі стека і виконуємо операцію
            let b = stack.pop().ok_or("Недостатньо чисел у виразі")?;
            let a = stack.pop().ok_or("Недостатньо чисел у виразі")?;

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
            return Err(format!("Непередбачений токен '{}'", token));
        }
    }

    // Якщо в стеку залишилося одне значення, це і є результат
    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else {
        Err("Неможливо обчислити вираз".to_string())
    }
}

// Функція для визначення пріоритету операторів
fn precedence(op: &str) -> i32 {
    match op {
        "+" | "-" => 1,
        "*" | "/" => 2,
        _ => 0,
    }
}

// Основна функція запуску Rocket серверу
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, calculate]) // Роутинг для головної сторінки та обчислень
        .mount("/static", rocket::fs::FileServer::from("static")) // Підключення статичних файлів
}
