use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, tuple},
    IResult,
};
#[derive(Debug)]
pub struct Bif {
    pub network: Network,
    pub variables: Vec<Variable>,
}

#[derive(Debug)]
pub struct Network {
    pub name: String,
    pub properties: Vec<Property>,
}
#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub node_type: VariableType,
    pub states: Vec<String>,
    pub properties: Vec<Property>,
}
#[derive(Debug)]
pub enum VariableType {
    Discrete(u32),
}
#[derive(Debug)]
pub struct Property {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct TableRow {
    pub key: Vec<String>,
    pub value: f64,
}

#[derive(Debug)]
pub struct Probability {
    pub variable_string: String,
    pub table: Vec<TableRow>,
    pub properties: Vec<Property>,
}

fn parse_name(input: &str) -> IResult<&str, String> {
    println!("parse_name: {}", input);
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        String::from,
    )(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), take_while(|c: char| c != '\"'), tag("\""))(input)?;
    Ok((input, String::from(input)))
}

fn parse_property(input: &str) -> IResult<&str, Property> {
    let (input, (key, value)) = tuple((parse_name, preceded(multispace0, parse_string)))(input)?;
    Ok((input, Property { key, value }))
}

fn parse_properties(input: &str) -> IResult<&str, Vec<Property>> {
    println!("parse_properties: {}", input);
    many0(preceded(multispace1, parse_property))(input)
}

fn parse_network(input: &str) -> IResult<&str, Network> {
    let (input, (_, name, properties)) = tuple((
        tag("network"),
        preceded(multispace1, parse_name),
        preceded(
            multispace0,
            delimited(tag("{"), parse_properties, preceded(multispace0, tag("}"))),
        ),
    ))(input)?;
    Ok((input, Network { name, properties }))
}

fn parse_node_type(input: &str) -> IResult<&str, VariableType> {
    let (input, node_type) = preceded(
        tag("type"),
        preceded(multispace0, take_while1(|c: char| c.is_alphabetic())),
    )(input)?;
    match node_type {
        "discrete" => {
            let (input, len) = (preceded(
                multispace0,
                delimited(
                    tag("["),
                    preceded(multispace0, parse_u32),
                    preceded(multispace0, tag("]")),
                ),
            )(input))?;
            println!("parse_node_type: {} {:?}", node_type, len);
            Ok((input, VariableType::Discrete(len)))
        }

        _ => Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Tag,
        })),
    }
}

fn parse_states(input: &str) -> IResult<&str, Vec<String>> {
    println!("parse_states: {}", input);
    separated_list0(tag(","), preceded(multispace0, parse_name))(input)
}

fn parse_probabilities(input: &str) -> IResult<&str, Vec<f64>> {
    separated_list0(tag(","), preceded(multispace0, parse_f64))(input)
}
fn parse_u32(input: &str) -> IResult<&str, u32> {
    let (input, num_str) = take_while1(|c: char| c.is_digit(10))(input)?;
    match num_str.parse::<u32>() {
        Ok(num) => Ok((input, num)),
        Err(_) => Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Digit,
        })),
    }
}
fn parse_f64(input: &str) -> IResult<&str, f64> {
    let (input, num_str) =
        take_while1(|c: char| c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '-')(
            input,
        )?;
    match num_str.parse::<f64>() {
        Ok(num) => Ok((input, num)),
        Err(_) => Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Float,
        })),
    }
}

fn parse_variable(input: &str) -> IResult<&str, Variable> {
    println!("parse_variable: {}", input);
    let (input, name) = preceded(tag("variable"), preceded(multispace1, parse_name))(input)?;
    println!("parse_variable: {}", name);
    let (input, (node_type, states, properties)) = preceded(
        multispace0,
        delimited(
            tag("{"),
            tuple((
                preceded(multispace0, parse_node_type),
                preceded(
                    multispace1,
                    delimited(tag("{"), parse_states, preceded(multispace0, tag("};"))),
                ),
                preceded(multispace1, parse_properties),
            )),
            preceded(multispace0, tag("}")),
        ),
    )(input)?;
    println!("parse_variable: {} {:?}", name, states);
    Ok((
        input,
        Variable {
            name,
            node_type,
            states,
            properties,
        },
    ))
}

fn parse_blocks(input: &str) -> IResult<&str, (Vec<Variable>, Vec<Probability>)> {
    todo!()
}

fn parse_bif(input: &str) -> IResult<&str, Bif> {
    let (input, (network, nodes)) = tuple((
        preceded(multispace0, parse_network),
        many0(preceded(multispace0, parse_variable)),
    ))(input)?;
    Ok((
        input,
        Bif {
            network,
            variables: nodes,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bif = r#"network unknown {
    }
    variable Difficulty {
    type discrete [ 2 ] { d0, d1 };
    }
    variable Intelligence {
    type discrete [ 2 ] { i0, i1 };
    }
    variable Grade {
    type discrete [ 3 ] { g0, g1, g2 };
    }
    variable Letter {
    type discrete [ 2 ] { l0, l1 };
    }
    variable SAT {
    type discrete [ 2 ] { s0, s1 };
    }
    probability ( Difficulty ) {
    table 0.6, 0.4;
    }
    probability ( Intelligence ) {
    table 0.7, 0.3;
    }
    probability ( Grade | Intelligence, Difficulty ) {
    (i0, d0) 0.30, 0.40, 0.30;
    (i0, d1) 0.05, 0.25, 0.70;
    (i1, d0) 0.90, 0.08, 0.02;
    (i1, d1) 0.50, 0.30, 0.20;
    }
    probability ( Letter | Grade ) {
    (g0) 0.10, 0.90;
    (g1) 0.40, 0.60;
    (g2) 0.99, 0.01;
    }
    probability ( SAT | Intelligence ) {
    (i0) 0.95, 0.05;
    (i1) 0.20, 0.80;
    }
        "#;
        //let result = add(2, 2);
        //assert_eq!(result, 4);
        let result = parse_bif(bif);
        println!("{:#?}", result);
    }
}
