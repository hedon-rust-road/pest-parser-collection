use std::collections::HashMap;

use anyhow::anyhow;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "json.pest"] // <- the path to your .pest file
struct JsonParser;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn main() -> anyhow::Result<()> {
    let s = r#"{
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "marks": [90.0, -80.0, 85.1],
        "address": {
            "city": "New York",
            "zip": 10001
        }
    }"#;

    let parsed = JsonParser::parse(Rule::json, s)?
        .next()
        .ok_or_else(|| anyhow!("json has no value"))?;

    let value = parse_value(parsed)?;
    println!("{:#?}", value);
    Ok(())
}

fn parse_array(pair: Pair<Rule>) -> anyhow::Result<Vec<JsonValue>> {
    pair.into_inner().map(parse_value).collect()
}

fn parse_object(pair: Pair<Rule>) -> anyhow::Result<HashMap<String, JsonValue>> {
    let inner = pair.into_inner();
    let values = inner.map(|pair| {
        let mut inner = pair.into_inner();
        let key = inner
            .next()
            .map(|p| p.as_str().to_string())
            .ok_or_else(|| anyhow!("expected key"))?;
        let value = parse_value(inner.next().ok_or_else(|| anyhow!("expected value"))?)?;
        Ok((key, value))
    });

    values.collect::<anyhow::Result<HashMap<_, _>>>()
}

fn parse_value(pair: Pair<Rule>) -> anyhow::Result<JsonValue> {
    let ret = match pair.as_rule() {
        Rule::chars => JsonValue::String(pair.as_str().parse()?),
        Rule::number => JsonValue::Number(pair.as_str().parse()?),
        Rule::bool => JsonValue::Bool(pair.as_str().parse()?),
        Rule::null => JsonValue::Null,
        Rule::array => JsonValue::Array(parse_array(pair)?),
        Rule::object => JsonValue::Object(parse_object(pair)?),
        Rule::value => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow!("expected value"))?;
            parse_value(inner)?
        }
        v => {
            panic!("unhandled rule: {:?}", v);
        }
    };
    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pest_parse_null_should_work() -> anyhow::Result<()> {
        let input = "null";
        let parsed = JsonParser::parse(Rule::null, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Null);
        Ok(())
    }

    #[test]
    fn pest_parse_bool_should_work() -> anyhow::Result<()> {
        let input = "true";
        let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Bool(true));

        let input = "false";
        let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Bool(false));

        Ok(())
    }

    #[test]
    fn pest_parse_number_should_work() -> anyhow::Result<()> {
        let input = "123.456";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Number(123.456));

        let input = "-123.456";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Number(-123.456));

        let input = "0";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Number(0.0));

        let input = "123";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Number(123.0));

        let input = "-123";
        let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
        assert_eq!(parse_value(parsed)?, JsonValue::Number(-123.0));

        Ok(())
    }

    #[test]
    fn pest_parse_string_should_work() -> anyhow::Result<()> {
        let input = r#""hello \" world\"""#;
        let parsed = JsonParser::parse(Rule::string, input)?.next().unwrap();
        assert_eq!(
            parse_value(parsed)?,
            JsonValue::String(r#"hello \" world\""#.to_string())
        );
        Ok(())
    }

    #[test]
    fn pest_parse_array_should_work() -> anyhow::Result<()> {
        let input = r#"["hello", "world"]"#;
        let parsed = JsonParser::parse(Rule::array, input)?.next().unwrap();
        assert_eq!(
            parse_value(parsed)?,
            JsonValue::Array(vec![
                JsonValue::String("hello".to_string()),
                JsonValue::String("world".to_string())
            ])
        );
        Ok(())
    }

    #[test]
    fn pest_parse_object_should_work() -> anyhow::Result<()> {
        let input = r#"{"hello": "world"}"#;
        let parsed = JsonParser::parse(Rule::object, input)?.next().unwrap();
        assert_eq!(
            parse_value(parsed)?,
            JsonValue::Object(
                vec![("hello".to_string(), JsonValue::String("world".to_string()))]
                    .into_iter()
                    .collect()
            )
        );
        Ok(())
    }
}
