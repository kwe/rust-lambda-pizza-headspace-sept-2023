use lambda_http::{run, http::{StatusCode, Response, header}, service_fn, Error, Body, IntoResponse, Request, RequestPayloadExt};
use serde::{Serialize, Deserialize};
use serde_json::json;

struct PizzaList {
    pizzas: Vec<Pizza>,
}
#[derive(Serialize)]
struct Pizza {
    name: String,
    price: u32,
}

impl PizzaList {
    fn new() -> PizzaList {
        PizzaList {
            pizzas: vec![
                Pizza { name: String::from("veggie"), price: 10 },
                Pizza { name: String::from("hawaiian"), price: 12 },
                Pizza { name: String::from("pepperoni"), price: 11 },
            ],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MyPayload {
    pub pizza: String,
}

fn get_pizza_from_name<'a>(pizza_name: &'a str, pizza_list: &'a PizzaList) -> Option<&'a Pizza> {
    let mut iter = pizza_list.pizzas.iter();
    iter.find(|pizza| pizza.name == pizza_name)
}

async fn build_success_response(pizza: &Pizza) -> Response<Body> {
    json!(pizza).into_response().await
}

async fn build_failure_response(error_message: &str) -> Response<Body> {
    Response::builder().status(StatusCode::BAD_REQUEST).header(header::CONTENT_TYPE, "application/json").body(Body::from(
    json!({ "error": error_message }).to_string())).expect("failed to render error response")
}

fn process_event<'a>(pizza_name: Option<&'a str>, pizza_list:&'a PizzaList) -> Result<&'a Pizza, &'a str> {
    match pizza_name {
        Some(name) => {
            match get_pizza_from_name(name, pizza_list) {
                Some(pizza) => Ok(pizza),
                None => {
                    Err("Pizza not found")
                },
            }
        },
        None => Err("No pizza name provided"),
    }
}

pub async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let pizza_list = PizzaList::new();

    if let Some(body) = event.payload::<MyPayload>().unwrap() {
        match process_event(Some(&body.pizza), &pizza_list) {
            Ok(pizza) => Ok(build_success_response(pizza).await),
            Err(e) => Ok(build_failure_response(e).await),
        }
    } else {
        Ok(build_failure_response("No payload provided").await)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_pizza_list_test() {
        let all_pizzas = PizzaList::new();
        assert_eq!(3, all_pizzas.pizzas.len());
        let veggie = get_pizza_from_name("veggie", &all_pizzas);
        let hawaiian = get_pizza_from_name("hawaiian", &all_pizzas);
        let pepperoni = get_pizza_from_name("pepperoni", &all_pizzas);
        assert_eq!(10, veggie.unwrap().price);
        assert_eq!(12, hawaiian.unwrap().price);
        assert_eq!(11, pepperoni.unwrap().price);
    }

    #[tokio::test]
    async fn build_success_response_test() {
        let test_pizza = Pizza { name: String::from("test"), price: 10 };
        let result = build_success_response(&test_pizza).await;
        let (parts, body) = result.into_parts(); 
        assert_eq!(200, parts.status);
        assert_eq!("application/json", parts.headers.get("content-type").unwrap().to_str().unwrap());
        assert_eq!("{\"name\":\"test\",\"price\":10}", String::from_utf8(body.to_ascii_lowercase()).unwrap());
    }
    #[tokio::test]
    async fn build_failure_response_test() {
        let result = build_failure_response("test error message.").await;
        let (parts, body) = result.into_parts(); 
        assert_eq!(400, parts.status);
        assert_eq!("application/json", parts.headers.get("content-type").unwrap().to_str().unwrap());
        assert_eq!("{\"error\":\"test error message.\"}", String::from_utf8(body.to_ascii_lowercase()).unwrap());
    }

    #[test]
    fn process_pizza_event_test() {
        let pizza_list = PizzaList::new();
        let res = process_event(Some("veggie"), &pizza_list);
        assert!(res.is_ok());
    }

    #[test]
    fn process_invalid_pizza_event_test() {
        let pizza_list = PizzaList::new();
        let res = process_event(Some("invalid"), &pizza_list);
        assert!(res.is_err());
    }

    #[test]
    fn process_no_pizza_event_test() {
        let pizza_list = PizzaList::new();
        let res = process_event(None, &pizza_list);
        assert!(res.is_err());
    }

}