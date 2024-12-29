use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{eof, map, map_res, value},
    multi::{many0, separated_list0},
    sequence::{delimited, terminated, tuple},
    IResult,
};
use serde::Serialize;

// AST Types with Serialize for JSON conversion
#[derive(Debug, Clone, Serialize)]
pub struct File {
    components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Component {
    Using(Using),
    Module(Module),
    Test(Test),
}

#[derive(Debug, Clone, Serialize)]
pub struct Using {
    type_sig: MType,
}

#[derive(Debug, Clone, Serialize)]
pub struct Module {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    gates: Vec<Gate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Test {
    name: String,
    type_sig: MType,
    patterns: Vec<TestPattern>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Gate {
    outputs: Vec<String>,
    module_name: String,
    inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MType {
    input_count: u32,
    output_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestPattern {
    inputs: Vec<bool>,
    outputs: Vec<bool>,
}


// Parser implementations
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_alphanumeric()),
        String::from,
    )(input)
}

fn right_arrow(input: &str) -> IResult<&str, &str> {
    alt((
        tag("->"),
        tag("=>"),
        tag(">"),
        tag("~>"),
    ))(input)
}

fn left_arrow(input: &str) -> IResult<&str, &str> {
    alt((
        tag("<-"),
        tag("<="),
        tag("<"),
        tag("<~"),
    ))(input)
}

fn using_keyword(input: &str) -> IResult<&str, &str> {
    alt((
        tag("using"),
        tag("Using"),
        tag("USING"),
        tag("use"),
        tag("Use"),
        tag("USE"),
    ))(input)
}

fn module_keyword(input: &str) -> IResult<&str, &str> {
    alt((
        tag("module"),
        tag("Module"),
        tag("MODULE"),
        tag("def"),
        tag("Def"),
        tag("DEF"),
    ))(input)
}

fn test_keyword(input: &str) -> IResult<&str, &str> {
    alt((
        tag("test"),
        tag("Test"),
        tag("TEST"),
    ))(input)
}

fn number(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

fn mtype(input: &str) -> IResult<&str, MType> {
    map(
        tuple((
            number,
            delimited(multispace0, right_arrow, multispace0),
            number,
        )),
        |(input_count, _, output_count)| MType {
            input_count,
            output_count,
        },
    )(input)
}

fn value_separator(input: &str) -> IResult<&str, ()> {
    alt((
        map(multispace1, |_| ()),
        map(
            tuple((
                multispace0,
                char(','),
                multispace0
            )),
            |_| ()
        )
    ))(input)
}

fn using(input: &str) -> IResult<&str, Using> {
    map(
        tuple((
            using_keyword,
            multispace0,
            tag("nor"),
            multispace0,
            char(':'),
            multispace0,
            tag("2"),
            multispace0,
            right_arrow,
            multispace0,
            tag("1"),
            multispace0,
            char(';'),
        )),
        |_| Using {
            type_sig: MType {
                input_count: 2,
                output_count: 1,
            },
        },
    )(input)
}

fn id_list(input: &str) -> IResult<&str, Vec<String>> {
    separated_list0(value_separator, identifier)(input)
}

fn io_list(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        char('('),
        delimited(multispace0, id_list, multispace0),
        char(')'),
    )(input)
}

fn gate_separator(input: &str) -> IResult<&str, &str> {
    alt((
        tag(":"),
        tag("="),
        tag(":="),
        tag("::="),
        left_arrow,
    ))(input)
}

fn gate(input: &str) -> IResult<&str, Gate> {
    map(
        tuple((
            id_list,
            multispace0,
            gate_separator,
            multispace0,
            identifier,
            multispace0,
            left_arrow,
            multispace0,
            id_list,
            multispace0,
            char(';'),
        )),
        |(outputs, _, _, _, module_name, _, _, _, inputs, _, _)| Gate {
            outputs,
            module_name,
            inputs,
        },
    )(input)
}

fn module(input: &str) -> IResult<&str, Module> {
    map(
        tuple((
            module_keyword,
            multispace0,
            identifier,
            multispace0,
            io_list,
            multispace0,
            right_arrow,
            multispace0,
            io_list,
            multispace0,
            delimited(
                char('{'),
                many0(delimited(multispace0, gate, multispace0)),
                char('}'),
            ),
        )),
        |(_, _, name, _, inputs, _, _, _, outputs, _, gates)| Module {
            name,
            inputs,
            outputs,
            gates,
        },
    )(input)
}

fn true_value(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, char('t')),
        value(true, char('T')),
        value(true, char('h')),
        value(true, char('H')),
        value(true, char('1')),
    ))(input)
}
fn false_value(input: &str) -> IResult<&str, bool> {
    alt((
        value(false, char('f')),
        value(false, char('F')),
        value(false, char('l')),
        value(false, char('L')),
        value(false, char('0')),
    ))(input)
}

fn bool_value(input: &str) -> IResult<&str, bool> {
    alt((
        true_value,
        false_value,
    ))(input)
}

fn bool_list(input: &str) -> IResult<&str, Vec<bool>> {
    separated_list0(value_separator, bool_value)(input)
}

fn test_pattern(input: &str) -> IResult<&str, TestPattern> {
    map(
        tuple((
            bool_list,
            multispace0,
            right_arrow,
            multispace0,
            bool_list,
            multispace0,
            char(';'),
        )),
        |(inputs, _, _, _, outputs, _, _)| TestPattern { inputs, outputs },
    )(input)
}

fn test(input: &str) -> IResult<&str, Test> {
    map(
        tuple((
            test_keyword,
            multispace0,
            identifier,
            multispace0,
            char(':'),
            multispace0,
            mtype,
            multispace0,
            delimited(
                char('{'),
                many0(delimited(multispace0, test_pattern, multispace0)),
                char('}'),
            ),
        )),
        |(_, _, name, _, _, _, type_sig, _, patterns)| Test {
            name,
            type_sig,
            patterns,
        },
    )(input)
}

fn component(input: &str) -> IResult<&str, Component> {
    alt((
        map(using, Component::Using),
        map(module, Component::Module),
        map(test, Component::Test),
    ))(input)
}

fn file(input: &str) -> IResult<&str, File> {
    map(
        terminated(
            many0(delimited(multispace0, component, multispace0)),
            tuple((multispace0, eof)),
        ),
        |components| File { components },
    )(input)
}

// Helper function to parse a string into our AST
pub fn parse(input: &str) -> Result<String, String> {
    match file(input) {
        Ok(("", ast)) => Ok(format!("{:?}",ast)),
        Ok((remainder, _)) => Err(format!("Parser did not consume all input. Remaining: {}", remainder)),
        Err(e) => Err(format!("Parser error: {}", e)),
    }
}