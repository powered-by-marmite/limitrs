use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CountRequest {
    pub direction: Direction,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseCountRequestError;

impl FromStr for CountRequest {
    type Err = ParseCountRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "incr" => Ok(CountRequest { direction: Direction::Increment }),
            "decr" => Ok(CountRequest { direction: Direction::Decrement }),
            _ => Err(ParseCountRequestError),
        }
    }
}

impl ToString for CountRequest {
    fn to_string(&self) -> String {
        match self.direction {
            Direction::Increment => "incr".to_owned(),
            Direction::Decrement => "decr".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CountResponse {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Direction {
    Increment,
    Decrement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_errors() {
        let s = "other";
        let result = CountRequest::from_str(s);
        let expected = Err(ParseCountRequestError);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_request_succeeds() {
        let s = "incr";
        let result = CountRequest::from_str(s);
        let expected = Ok(CountRequest { direction: Direction::Increment });
        assert_eq!(result, expected);

        let s = "decr";
        let result = CountRequest::from_str(s);
        let expected = Ok(CountRequest { direction: Direction::Decrement });
        assert_eq!(result, expected);
    }

    #[test]
    fn to_string_outputs_expected() {
        assert_eq!("incr", CountRequest { direction: Direction::Increment }.to_string());
        assert_eq!("decr", CountRequest { direction: Direction::Decrement }.to_string());
    }

}