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

fn hex_digit(input: &str) -> IResult<&str, char> {
    alt((
        char('0'),
        char('1'),
        char('2'),
        char('3'),
        char('4'),
        char('5'),
        char('6'),
        char('7'),
        char('8'),
        char('9'),
        char('a'),
        char('b'),
        char('c'),
        char('d'),
        char('e'),
        char('f'),
    ))(input)
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

fn graphical_keyword(input: &str) -> IResult<&str, &str> {
    alt((
        tag("graphical"),
        tag("Graphical"),
        tag("GRAPHICAL"),
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

fn img_size_auto(input: &str) -> IResult<&str, ImgSize> {
    map(
        tag("auto"),
        |_| ImgSize::Auto(()),
    )(input)
}

fn img_size_number(input: &str) -> IResult<&str, ImgSize> {
    map(
        tuple((
            number,
            char('x'),
            number,
        )),
        |(width,_,height)| ImgSize::Size((width,height)),
    )(input)
}

/// for img_color parser
fn hex_to_u8(a: char, b: char) -> Option<u8> {
    match (a.to_digit(16), b.to_digit(16)) {
        (Some(x), Some(y)) if x <= 15 && y <= 15 => Some(TryFrom::try_from(x * 16 + y).unwrap()),
        _ => None
    }
}

fn img_color(input: &str) -> IResult<&str, (u8,u8,u8)> {
    map(
        tuple((
            char('#'),
            hex_digit,
            hex_digit,
            hex_digit,
            hex_digit,
            hex_digit,
            hex_digit,
        )),
        |(_,a,b,c,d,e,f)| (hex_to_u8(a,b).unwrap(),hex_to_u8(c,d).unwrap(),hex_to_u8(e,f).unwrap())
    )(input)
}

fn graphical(input: &str) -> IResult<&str, Graphical> {
    map(
        tuple((
            graphical_keyword,
            multispace0,
            identifier,
            multispace0,
            char(':'),
            multispace0,
            alt((img_size_auto,img_size_number)),
            multispace0,
            delimited(
                char('{'),
                many0(delimited(separator, pixel, separator)),
                char('}'),
            ),
        )),
        |(_, _,name,_,_,_,size, _, pixels)| Graphical {
            name,
            size,
            pixels,
        },
    )(input)
}

fn img_io_name(input: &str) -> IResult<&str, IoIndex> {
    map(
        tuple((
            alt((char('i'),char('o'))),
            number,
        )),
        |(io_type,index)| IoIndex {
            io_type: match io_type {
                'i' => "input".to_string(),
                _ => "output".to_string(),
            },
            index,
        }
    )(input)
}


fn pixel(input: &str) -> IResult<&str, Pixel> {
    map(
        tuple((
            number,
            value_separator,
            number,
            multispace0,
            gate_separator,
            multispace0,
            img_io_name,
            multispace0,
            opt(tuple((left_arrow, multispace0))),
            img_color,
            value_separator,
            img_color,
            char(';'),
        )),
        |(x,_,y,_,_,_,io_index,_,_,color_on,_,color_off,_)| Pixel {
            coord: (x,y),
            io_index,
            color: PixelColor {
                on: color_on,
                off: color_off,
            }
        },
    )(input)
}


fn component(input: &str) -> IResult<&str, Component> {
    alt((
        map(using, Component::Using),
        // map(import, Component::Import),
        map(module, Component::Module),
        map(graphical, Component::Graphical),
        map(func_module, Component::Module),
        map(test, Component::Test),
        map(include, Component::Include),
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