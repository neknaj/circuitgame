use super::types::*;
use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, not_line_ending},
    combinator::{eof, map, map_res, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, terminated, tuple, preceded},
    IResult,
};

// Parser implementations


fn identifier(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        String::from,
    )(input)
}

fn natural_number(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
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


fn id_list_output(input: &str) -> IResult<&str, Vec<PreOutputs>> {
    separated_list0(value_separator, array_declaration)(input)
}
fn id_list_input(input: &str) -> IResult<&str, Vec<PreInputs>> {
    separated_list0(value_separator, array_slice)(input)
}

fn io_list_input(input: &str) -> IResult<&str, Vec<PreOutputs>> {
    delimited(
        char('('),
        delimited(multispace0, separated_list0(value_separator, array_declaration), multispace0),
        char(')')
    )(input)
}

fn io_list_output(input: &str) -> IResult<&str, Vec<PreInputs>> {
    delimited(
        char('('),
        delimited(multispace0, separated_list0(value_separator, array_slice), multispace0),
        char(')')
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

// 配列の宣言をパースします：
//  - "identifier"        -> これは identifier(1) と解釈されます。
//  - "identifier(n)"     -> nは自然数。
fn array_declaration(input: &str) -> IResult<&str, PreOutputs> {
    let (input, arr_name) = identifier(input)?;
    let (input, arr_size_) = opt(delimited(char('('), natural_number, char(')')))(input)?;
    let arr_size = arr_size_.unwrap_or_else(|| 1);
    Ok((input,PreOutputs{arr_name,arr_size}))
    // Ok((input,(0..size).map(|i| format!("{}:{}",arr_name,i)).collect()))
}

// スライス記法をパースします。
// 記法は以下の通りです:
//   - [x,y]  : 閉区間 (xもyも含む)
//   - (x,y)  : 開区間 (xもyも含まない)
//   - [x,y)  : 左側閉、右側開
//   - (x,y]  : 左側開、右側閉
// さらに、1つの数字の場合は id[x] として id[x,x] とみなします。
fn slice_specifier(input: &str) -> IResult<&str, ArrSlice> {
    // 開き括弧のパース: '[' または '('
    let (input, opening) = alt((char('['), char('(')))(input)?;
    let lower_inclusive = opening == '[';

    // 最初の数字をパース
    let (input, first_num) = natural_number(input)?;

    // オプションでカンマと第二の数字をパース
    let (input, second_opt) = opt(preceded(char(','), natural_number))(input)?;

    // 閉じ括弧のパース: ']' または ')'
    let (input, closing) = alt((char(']'), char(')')))(input)?;
    let upper_inclusive = closing == ']';

    // 第二の数字がなければ、最初の数字を使用 (id[x] == id[x,x])
    let second_num = second_opt.unwrap_or(first_num);

    let slice = ArrSlice {
        all: false,
        start: first_num,
        end: second_num,
        lower_inclusive,
        upper_inclusive,
    };
    Ok((input, slice))
}

// 配列のスライス記法をパースする関数です。
// サポートする記法は:
//   - id[x,y]  (両端含む)
//   - id(x,y)  (両端含まない)
//   - id[x,y)  (左端含む、右端含まない)
//   - id(x,y]  (左端含まない、右端含む)
//   - id[x]    (省略形: id[x,x])
//   - id       (スライスが指定されない場合は id[0,0] とみなす)
fn array_slice(input: &str) -> IResult<&str, PreInputs> {
    let (input, id_str) = identifier(input)?;
    // オプションでスライス記法をパース
    let (input, slice_opt) = opt(slice_specifier)(input)?;
    // スライス記法がない場合はデフォルトで [0,0] (両端含む) とする
    let slice = slice_opt.unwrap_or(ArrSlice {
        all: true,
        start: 0,
        end: 0,
        lower_inclusive: true,
        upper_inclusive: true,
    });
    Ok((input, PreInputs {
        arr_name: id_str,
        arr_slice: slice,
    }))
}

fn gate(input: &str) -> IResult<&str, PreGate> {
    map(
        tuple((
            id_list_output,
            multispace0,
            gate_separator,
            multispace0,
            identifier,
            multispace0,
            opt(tuple((left_arrow, multispace0))),
            id_list_input,
            multispace0,
            char(';'),
        )),
        |(outputs, _, _, _, module_name, _, _, inputs, _, _)| PreGate {
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
            io_list_input,
            multispace0,
            right_arrow,
            multispace0,
            io_list_output,
            separator,
            alt((
                delimited(
                    char('{'),
                    many0(delimited(separator, gate, separator)),
                    char('}'),
                ),
                map(
                    tuple((
                        char('{'),
                        separator,
                        char('}'),
                    )),
                    |_| Vec::new()
                )
            )),
        )),
        |(_, _, name, _, inputs_pre, _, _, _, outputs_pre, _, gates_pre)| {
            let (inputs,outputs,gates) = convert_pre_gates(inputs_pre,outputs_pre,gates_pre);
            Module {
                func: false,
                name,
                inputs,
                outputs,
                gates: gates,
            }
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
            io_list_input,
            multispace0,
            right_arrow,
            multispace0,
            io_list_output,
            separator,
            alt((
                delimited(
                    char('{'),
                    many0(delimited(separator, gate, separator)),
                    char('}'),
                ),
                map(
                    tuple((
                        char('{'),
                        separator,
                        char('}'),
                    )),
                    |_| Vec::new()
                )
            )),
        )),
        |(_, _, name, _, inputs_pre, _, _, _, outputs_pre, _, gates_pre)| {
            let (inputs,outputs,gates) = convert_pre_gates(inputs_pre,outputs_pre,gates_pre);
            Module {
                func: true,
                name,
                inputs,
                outputs,
                gates: gates,
            }
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
            alt((
                delimited(
                    char('{'),
                    many0(delimited(separator, test_pattern, separator)),
                    char('}'),
                ),
                map(
                    tuple((
                        char('{'),
                        separator,
                        char('}'),
                    )),
                    |_| Vec::new()
                )
            )),
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
        |(width,_,height)| ImgSize::Size {
            width,
            height,
        },
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
            alt((
                delimited(
                    char('{'),
                    many0(delimited(separator, pixel, separator)),
                    char('}'),
                ),
                map(
                    tuple((
                        char('{'),
                        separator,
                        char('}'),
                    )),
                    |_| Vec::new()
                )
            )),
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
        Ok((remainder, _)) => {
            // Find the context of the error
            let error_pos = input.len() - remainder.len();
            let start = input[..error_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let end = remainder.find('\n').map(|i| error_pos + i).unwrap_or(input.len());
            let line = input[start..end].trim();
            // Calculate line number
            let line_number = input[..error_pos].matches('\n').count() + 1;
            // Create detailed error message
            let mut error = format!("Parsing error at line {}:\n", line_number);
            error.push_str(&format!("{}\n", line));
            // Add pointer to the error position
            error.push_str(&format!("{}^\n", " ".repeat(error_pos - start)));
            error.push_str("Unexpected content found. The parser was unable to continue from this point.\n");
            error.push_str("Common causes:\n");
            error.push_str("- Missing semicolon at the end of a statement\n");
            error.push_str("- Invalid syntax or typo in module/gate definition\n");
            error.push_str("- Unmatched braces or parentheses\n");
            Err(error)
        },
        Err(e) => {
            let error_desc = match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => {
                    // Convert nom error into more user-friendly message
                    let pos = input.len() - e.input.len();
                    let start = input[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let end = e.input.find('\n').map(|i| pos + i).unwrap_or(input.len());
                    let line = input[start..end].trim();
                    let line_number = input[..pos].matches('\n').count() + 1;
                    format!("Syntax error at line {}:\n{}\n{}^\nInvalid syntax found here.\n",
                        line_number,
                        line,
                        " ".repeat(pos - start)
                    )
                },
                nom::Err::Incomplete(_) => "Incomplete input: the file appears to be truncated.".to_string(),
            };
            Err(error_desc)
        },
    }
}

/// Convert pre-module and gate data into final representations.
///
/// # Parameters
/// - `module_inputs_pre`: PreOutputs for module-level inputs.
/// - `module_outputs_pre`: PreInputs for module-level outputs.
/// - `gates_pre`: PreGate data to be converted into Gate structures.
///
/// # Returns
/// A tuple containing:
/// - Converted module inputs as Vec<String>
/// - Converted module outputs as Vec<String>
/// - Converted gates as Vec<Gate>
pub fn convert_pre_gates(
    module_inputs_pre: Vec<PreOutputs>,
    module_outputs_pre: Vec<PreInputs>,
    gates_pre: Vec<PreGate>,
) -> (Vec<String>, Vec<String>, Vec<Gate>) {
    // Step 1: Build the output_sizes mapping using module_inputs_pre and gates_pre outputs.
    let mut output_sizes: HashMap<String, usize> = HashMap::new();
    // Insert sizes from module_inputs_pre.
    for po in &module_inputs_pre {
        output_sizes.insert(po.arr_name.clone(), po.arr_size);
    }
    // Insert sizes from each gate's outputs.
    for gate in &gates_pre {
        for po in &gate.outputs {
            // Only insert if not already present.
            output_sizes.entry(po.arr_name.clone()).or_insert(po.arr_size);
        }
    }

    // Step 2: Convert module_inputs_pre into a vector of strings.
    let module_inputs: Vec<String> = module_inputs_pre
        .into_iter()
        .flat_map(|po| {
            (0..po.arr_size)
                .map(|i| format!("{}:{}", po.arr_name, i))
                .collect::<Vec<String>>()
        })
        .collect();

    // Convert each gate's outputs and inputs.
    let gates: Vec<Gate> = gates_pre
        .into_iter()
        .map(|pre_gate| {
            // Convert outputs using the output_sizes mapping.
            let outputs: Vec<String> = pre_gate
                .outputs
                .into_iter()
                .flat_map(|po| {
                    // Retrieve size from the mapping.
                    let size = *output_sizes.get(&po.arr_name).unwrap_or(&po.arr_size);
                    (0..size)
                        .map(|i| format!("{}:{}", po.arr_name, i))
                        .collect::<Vec<String>>()
                })
                .collect();

            // Convert inputs using the output_sizes mapping.
            let inputs: Vec<String> = pre_gate
                .inputs
                .into_iter()
                .flat_map(|pi| {
                    let name = pi.arr_name;
                    let slice = pi.arr_slice;
                    // Determine the effective size from output_sizes.
                    let size = *output_sizes.get(&name).unwrap_or(&100);
                    let (lower, upper) = if slice.all {
                        if size > 0 {
                            (0, size - 1)
                        } else {
                            (0, 0)
                        }
                    } else {
                        let lower = if slice.lower_inclusive { slice.start } else { slice.start + 1 };
                        let upper = if slice.upper_inclusive { slice.end } else { slice.end - 1 };
                        (lower, upper)
                    };
                    if lower <= upper {
                        (lower..=upper)
                            .map(|i| format!("{}:{}", name, i))
                            .collect::<Vec<String>>()
                    } else {
                        vec![]
                    }
                })
                .collect();

            Gate {
                module_name: pre_gate.module_name,
                outputs,
                inputs,
            }
        })
        .collect();

    // Step 3: Convert module_outputs_pre into a vector of strings using output_sizes.
    let module_outputs: Vec<String> = module_outputs_pre
        .into_iter()
        .flat_map(|pi| {
            let name = pi.arr_name;
            let slice = pi.arr_slice;
            // Retrieve size from the mapping.
            let size = *output_sizes.get(&name).unwrap_or(&0);
            let (lower, upper) = if slice.all {
                if size > 0 {
                    (0, size - 1)
                } else {
                    (0, 0)
                }
            } else {
                let lower = if slice.lower_inclusive { slice.start } else { slice.start + 1 };
                let upper = if slice.upper_inclusive { slice.end } else { slice.end - 1 };
                (lower, upper)
            };
            if lower <= upper {
                (lower..=upper)
                    .map(|i| format!("{}:{}", name, i))
                    .collect::<Vec<String>>()
            } else {
                vec![]
            }
        })
        .collect();

    (module_inputs, module_outputs, gates)
}