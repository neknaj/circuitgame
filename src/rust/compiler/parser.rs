use super::types::*;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, not_line_ending},
    combinator::{eof, map, map_res, value, recognize, opt},
    multi::{many0, separated_list0},
    sequence::{delimited, terminated, tuple},
    IResult,
};

// Parser implementations
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        String::from,
    )(input)
}

fn file_path_1(input: &str) -> IResult<&str, String> {
    map(
        delimited(char('"'), take_while1(|c| c != '"'), char('"')),
        String::from,
    )(input)
}

fn file_path_2(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| !c.is_whitespace() && c != ';'),
        String::from,
    )(input)
}

fn file_path(input: &str) -> IResult<&str, String> {
    alt((
        file_path_1,
        file_path_2,
    ))(input)
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

fn include_keyword(input: &str) -> IResult<&str, &str> {
    alt((
        tag("include"),
        tag("Include"),
        tag("INCLUDE"),
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

fn func_keyword(input: &str) -> IResult<&str, &str> {
    alt((
        tag("func"),
        tag("Func"),
        tag("FUNC"),
        tag("fn"),
        tag("Fn"),
        tag("FN"),
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
            input_count: input_count as usize,
            output_count: output_count as usize,
        },
    )(input)
}


fn line_comment_start(input: &str) -> IResult<&str, &str> {
    alt((
        tag("//"),
        tag("#"),
    ))(input)
}

fn line_comment(input: &str) -> IResult<&str, &str> {
    recognize(
        tuple((
            line_comment_start,
            not_line_ending,
            alt((
                tag("\n"),
                tag("\r\n"),
                eof
            ))
        ))
    )(input)
}

/// コメント有りの区切り
fn separator(input: &str) -> IResult<&str, ()> {
    map(
        many0(
            alt((
                map(multispace1, |_| ()),
                map(line_comment, |_| ())
            ))
        ),
        |_| ()
    )(input)
}

/// valueの区切り
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

fn include(input: &str) -> IResult<&str, Include> {
    map(
        tuple((
            char('!'),
            include_keyword,
            multispace0,
            file_path,
            char(';'),
        )),
        |(_,_,_,path,_)| Include {
            path,
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
            opt(tuple((left_arrow, multispace0))),
            id_list,
            multispace0,
            char(';'),
        )),
        |(outputs, _, _, _, module_name, _, _, inputs, _, _)| Gate {
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
            separator,
            delimited(
                char('{'),
                many0(delimited(separator, gate, separator)),
                char('}'),
            ),
        )),
        |(_, _, name, _, inputs, _, _, _, outputs, _, gates)| Module {
            func: false,
            name,
            inputs,
            outputs,
            gates,
        },
    )(input)
}

fn func_module(input: &str) -> IResult<&str, Module> {
    map(
        tuple((
            func_keyword,
            multispace0,
            identifier,
            multispace0,
            io_list,
            multispace0,
            right_arrow,
            multispace0,
            io_list,
            separator,
            delimited(
                char('{'),
                many0(delimited(separator, gate, separator)),
                char('}')),
        )),
        |(_, _, name, _, inputs, _, _, _, outputs, _, gates)| Module {
            func: true,
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
            separator,
            delimited(
                char('{'),
                many0(delimited(separator, test_pattern, separator)),
                char('}')),
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
        map(include, Component::Include),
        map(using, Component::Using),
        map(module, Component::Module),
        map(func_module, Component::Module),
        map(test, Component::Test),
    ))(input)
}

fn file(input: &str) -> IResult<&str, File> {
    map(
        terminated(
            many0(delimited(separator, component, separator)),
            tuple((separator, eof)),
        ),
        |components| File { components },
    )(input)
}


pub fn parser(input: &str) -> Result<File, String> {
    match file(input) {
        Ok(("", ast)) => Ok(ast),
        Ok((remainder, _)) => Err(format!("Remaining: {}", remainder)),
        Err(e) => Err(format!("{}", e)),
    }
}