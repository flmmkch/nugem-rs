use ::game::input;
use nom::{ErrorKind, IResult, digit, space};
use std::str;
use super::{CommandInputState, ModifiedInput};

pub fn parse_command_input(command_string: &str) -> Result<Vec<CommandInputState>, String> {
    let result = parse_full(command_string.as_bytes());
    println!("{:?}", &result);
    match result {
        IResult::Done(_, mut command_states) => {
            // before sending back the command states: expand the directions
            let mut i = 0;
            if command_states.len() > 0 {
                while i < command_states.len() - 1 {
                    // as explained in the cmd file of the example character KFM:
                    // "Successive direction symbols are always expanded in a manner similar to this example:
                    // command = F, F
                    // is expanded when MUGEN reads it, to become equivalent to:
                    // command = F, >~F, >F"
                    if let Some(ModifiedInput::Normal(directional)) = command_states[i].directional.clone() {
                        // if the following command input also has a direction symbol
                        if let Some(ModifiedInput::Normal(next_directional)) = command_states[i+1].directional.clone() {
                            if &next_directional == &directional {
                                command_states[i+1].strict = true;
                                let mut inserted_directional = CommandInputState::new();
                                inserted_directional.strict = true;
                                inserted_directional.directional = Some(ModifiedInput::Release(directional.clone(), None));
                                command_states.insert(i+1, inserted_directional);
                            }
                        }
                    }
                    i += 1;
                }
            }
            Ok(command_states)
        },
        _ => Err(command_string.to_owned()),
    }
}

named!(parse_full<&[u8], Vec<CommandInputState>>,
    separated_nonempty_list_complete!(
        tag!(","),
        do_parse!(
            opt!(space) >>
            input_state: parse_input_state >>
            (input_state)
        )
    )
);

named!(parse_input_state<&[u8], CommandInputState>,
    map!(
        do_parse!(
            strict: opt!(tag!(">")) >>
            partial_state: complete!(parse_symbols) >>
            (strict, partial_state)
        ),
        |(strict_option, mut partial_state)| -> CommandInputState {
            if let Some(_) = strict_option {
                partial_state.strict = true;
            }
            partial_state
        }
    )
);

named!(parse_symbols<&[u8], CommandInputState>,
    map!(
        separated_nonempty_list_complete!(
            tag!("+"),
            complete!(parse_modified_symbol)
        ),
        |input_states| -> CommandInputState {
            // merge partial states
            let mut input_state = CommandInputState::new();
            for added_state in input_states {
                input_state.button_presses.extend(added_state.button_presses);
                if let Some(directional) = added_state.directional {
                    input_state.directional = Some(directional);
                }
            }
            input_state
        }
    )
);

named!(release_modifier<&[u8], Option<u16>>,
    do_parse!(
        tag!("~") >>
        time_opt: 
            map!(
                opt!(
                    digit
                ),
                |digits_opt: Option<&[u8]>| -> Option<u16> {
                    if let Some(digits) = digits_opt {
                        if let Ok(string) = str::from_utf8(digits) {
                            if let Ok(number) = string.parse() {
                                Some(number)
                            }
                            else {
                                None
                            }
                        }
                        else {
                            None
                        }
                    }
                    else {
                        None
                    }
                }
            ) >>
        (time_opt)
    )
);

named!(parse_normal_button<&[u8], CommandInputState>,
    map!(
        parse_button_symbol,
        |button: input::Button| -> CommandInputState {
            let mut input_state = CommandInputState::new();
            input_state.button_presses.push(ModifiedInput::Normal(button));
            input_state
        }
    )
);

named!(parse_held_down_button<&[u8], CommandInputState>,
    map!(
        do_parse!(
            tag!("/") >>
            button: parse_button_symbol >>
            (button)
        ),
        |button: input::Button| -> CommandInputState {
            let mut input_state = CommandInputState::new();
            input_state.button_presses.push(ModifiedInput::HoldDown(button));
            input_state
        }
    )
);

named!(parse_released_button<&[u8], CommandInputState>,
    map!(
        do_parse!(
            time_opt: release_modifier >>
            button: parse_button_symbol >>
            (button, time_opt)
        ),
        |(button, time_opt): (input::Button, Option<u16>)| -> CommandInputState {
            let mut input_state = CommandInputState::new();
            input_state.button_presses.push(ModifiedInput::Release(button, time_opt));
            input_state
        }
    )
);

named!(parse_normal_direction<&[u8], CommandInputState>,
    map!(
        parse_directional_input,
        |directional: input::PartialDirectional| -> CommandInputState {
            let mut input_state = CommandInputState::new();
            input_state.directional = Some(ModifiedInput::Normal(directional));
            input_state
        }
    )
);

named!(parse_held_down_direction<&[u8], CommandInputState>,
    map!(
        do_parse!(
            tag!("/") >>
            directional: parse_directional_input >>
            (directional)
        ),
        |directional: input::PartialDirectional| -> CommandInputState {
            let mut input_state = CommandInputState::new();
            input_state.directional = Some(ModifiedInput::HoldDown(directional));
            input_state
        }
    )
);

named!(parse_released_direction<&[u8], CommandInputState>,
    map!(
        do_parse!(
            time_opt: release_modifier >>
            directional: parse_directional_input >>
            (directional, time_opt)
        ),
        |(directional, time_opt): (input::PartialDirectional, Option<u16>)| -> CommandInputState {
            let mut input_state = CommandInputState::new();
            input_state.directional = Some(ModifiedInput::Release(directional, time_opt));
            input_state
        }
    )
);

named!(parse_modified_symbol<&[u8], CommandInputState>,
    alt!(
        // held down button
        complete!(parse_held_down_button) |
        // released button
        complete!(parse_released_button) |
        // held down direction
        complete!(parse_held_down_direction) |
        // released direction
        complete!(parse_released_direction) |
        // normal direction
        complete!(parse_normal_direction) |
        // normal button
        complete!(parse_normal_button)
    )
);

macro_rules! value_from_tag {
    ($i: expr, $input: expr, $button: expr) => {
        value!(
            $i,
            $button,
            tag!($input)
        )
    };
}

named!(parse_button_symbol<&[u8], input::Button>,
    alt!(
        value_from_tag!("a", input::Button::A) |
        value_from_tag!("b", input::Button::B) |
        value_from_tag!("c", input::Button::C) |
        value_from_tag!("x", input::Button::X) |
        value_from_tag!("y", input::Button::Y) |
        value_from_tag!("z", input::Button::Z)
    )
);

named!(parse_directional_input<&[u8], input::PartialDirectional>,
    alt!(
        value_from_tag!("$U", input::PartialDirectional::Vertical(input::DirectionState::Plus)) |
        value_from_tag!("$D", input::PartialDirectional::Vertical(input::DirectionState::Minus)) |
        value_from_tag!("$F", input::PartialDirectional::Horizontal(input::DirectionState::Plus)) |
        value_from_tag!("$B", input::PartialDirectional::Horizontal(input::DirectionState::Minus)) |
        complete!(
            alt!(
                value_from_tag!("UF", input::PartialDirectional::FullDirection(input::Directional::UpForward)) |
                value_from_tag!("UB", input::PartialDirectional::FullDirection(input::Directional::Backward)) |
                value_from_tag!("DF", input::PartialDirectional::FullDirection(input::Directional::DownForward)) |
                value_from_tag!("DB", input::PartialDirectional::FullDirection(input::Directional::DownBackward))
            )
        ) |
        value_from_tag!("U", input::PartialDirectional::FullDirection(input::Directional::Up)) |
        value_from_tag!("F", input::PartialDirectional::FullDirection(input::Directional::Forward)) |
        value_from_tag!("D", input::PartialDirectional::FullDirection(input::Directional::Down)) |
        value_from_tag!("B", input::PartialDirectional::FullDirection(input::Directional::Backward))
    )
);

#[cfg(test)]
mod test {
    use super::*;
    // testing the parser
    // first test only bits of parsing expressions
    #[test]
    fn parser_test_bits() {
        {
            let (_, result) = parse_button_symbol(&b"c"[..]).unwrap();
            assert_eq!(result, input::Button::C);
        }
        {
            let (_, result) = parse_button_symbol(&b"x"[..]).unwrap();
            assert_eq!(result, input::Button::X);
        }
        {
            let (_, result) = parse_directional_input(&b"D"[..]).unwrap();
            assert_eq!(result, input::PartialDirectional::FullDirection(input::Directional::Down));
        }
        {
            let (_, result) = parse_directional_input(&b"DF"[..]).unwrap();
            assert_eq!(result, input::PartialDirectional::FullDirection(input::Directional::DownForward));
        }
        {
            let test_result = parse_modified_symbol(&b"~D"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], CommandInputState {
                directional: Some(ModifiedInput::Release(input::PartialDirectional::FullDirection(input::Directional::Down), None)),
                button_presses: Vec::new(),
                strict: false
            }));
        }
        {
            let test_result = parse_normal_button(&b"x"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false
            }));
        }
        {
            let test_result = parse_modified_symbol(&b"x"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false
            }));
        }
        {
            let test_result = parse_symbols(&b"x"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false
            }));
        }
        {
            let test_result = parse_input_state(&b"x"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false
            }));
        }
        {
            let test_result = parse_input_state(&b">x"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: true
            }));
        }
        {
            let test_result = parse_full(&b">x"[..]);
            assert_eq!(test_result, IResult::Done(&b""[..], vec![CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: true
            }]));
        }
    }
    #[test]
    fn parser_test_x() {
        let parsing_result = parse_command_input("x");
        assert_eq!(parsing_result, Ok(vec![CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false
        }]));
        let parsing_result = parse_command_input(" x");
        assert_eq!(parsing_result, Ok(vec![CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false
        }]));
    }
    // quarter circle forward + x
    #[test]
    fn parser_test_qcf_x() {
        let parsing_result = parse_command_input("~D, DF, F, x");
        assert!(parsing_result.is_ok());
        let command_states = parsing_result.unwrap();
        assert_eq!(command_states,
            vec![CommandInputState {
                directional: Some(ModifiedInput::Release(input::PartialDirectional::FullDirection(input::Directional::Down), None)),
                button_presses: Vec::new(),
                strict: false },
                CommandInputState {
                directional: Some(ModifiedInput::Normal(input::PartialDirectional::FullDirection(input::Directional::DownForward))),
                button_presses: Vec::new(),
                strict: false },
                CommandInputState {
                directional: Some(ModifiedInput::Normal(input::PartialDirectional::FullDirection(input::Directional::Forward))),
                button_presses: Vec::new(),
                strict: false },
                CommandInputState {
                directional: None,
                button_presses: vec![ModifiedInput::Normal(input::Button::X)],
                strict: false }
            ]
        );
    }
}
