use crate::game::{input, mugen::character::command::{InputModifier, InputSymbol}};
use nom::{IResult, multi::separated_list1, sequence::{preceded, tuple, terminated}, character::complete::{space0, digit1}, combinator::{map, value, opt, complete}, bytes::complete::tag, branch::alt, error::Error, Finish};
use std::str;
use super::CommandInputState;

pub fn parse_command_input(command_string: &str) -> Result<Vec<CommandInputState>, Error<&str>> {
    let (_, mut command_states) = complete(parse_list)(command_string).finish()?;
    log::trace!("Parsed command string \"{command_string}\": {command_states:?}");
    let mut i = 0;
    if command_states.len() > 0 {
        while i + 1 < command_states.len() {
            // as explained in the cmd file of the example character KFM:
            // "Successive direction symbols are always expanded in a manner similar to this example:
            // command = F, F
            // is expanded when MUGEN reads it, to become equivalent to:
            // command = F, >~F, >F"
            fn expand_after_direction_modifier(modifier: InputModifier) -> bool {
                modifier == InputModifier::Normal || modifier == InputModifier::HoldDown
            }
            match (&command_states[i].symbol, command_states[i].modifier) {
                (InputSymbol::Direction(direction), modifier) if expand_after_direction_modifier(modifier) => {
                    match (&command_states[i + 1].symbol, command_states[i + 1].modifier) {
                        (InputSymbol::Direction(_), next_modifier) if expand_after_direction_modifier(next_modifier) => {
                            // insert the input release
                            let mut inserted_directional_input = CommandInputState::from(InputSymbol::Direction(*direction));
                            inserted_directional_input.strict = true;
                            inserted_directional_input.modifier = InputModifier::Release { time: None };
                            // next input specified becomes strict
                            command_states[i + 1].strict = true;
                            // insert the input expansion release
                            command_states.insert(i + 1, inserted_directional_input);
                            i += 1;
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
            i += 1;
        }
    }
    Ok(command_states)
}

fn parse_list(input: &str) -> IResult<&str, Vec<CommandInputState>> {
    separated_list1(
    tag(","),
    preceded(space0, parse_input_state)
    )(input)
}

fn parse_input_state(input: &str) -> IResult<&str, CommandInputState> {
    map(
        tuple((
            map(
                opt(tag(">")),
                |o| o.is_some(),
            ),
            opt(parse_release_modifier),
            map(
                opt(tag("/")),
                |o| o.is_some(),
            ),
            map(
                opt(tag("$")),
                |o| o.is_some(),
            ),
            parse_input_symbol,
        )),
        |(strict, release, hold_down, partial, input_symbol)| {
            let mut input_state = CommandInputState::from(input_symbol);
            input_state.strict = strict;
            if hold_down {
                input_state.modifier = InputModifier::HoldDown;
            } else if let Some(time) = release {
                input_state.modifier = InputModifier::Release { time };
            }
            input_state.partial = partial;
            input_state
        }
    )
    (input)
}

fn parse_input_symbol(input: &str) -> IResult<&str, InputSymbol> {
    alt((
        // buttons
        map(
            separated_list1(tag("+"), preceded(space0, terminated(parse_button_symbol, space0))),
            |buttons| InputSymbol::from_iter(buttons.into_iter())
        ),
        // direction
        map(parse_directional_input, InputSymbol::from)
    ))
    (input)
}

fn parse_release_modifier(input: &str) -> IResult<&str, Option<u16>>
{
    preceded(
        tag("~"),
        map(
            opt(digit1),
            |digits_opt| digits_opt.and_then(|s: &str| s.parse().ok())
        )
    )
    (input)
}

fn parse_button_symbol(input: &str) -> IResult<&str, input::Button>
{
    alt((
        value(input::Button::A, tag("a")),
        value(input::Button::B, tag("b")),
        value(input::Button::C, tag("c")),
        value(input::Button::X, tag("x")),
        value(input::Button::Y, tag("y")),
        value(input::Button::Z, tag("z")),
        value(input::Button::Start, tag("s")),
    ))
    (input)
}

fn parse_directional_input(input: &str) -> IResult<&str, input::Directional>
{
    alt((
        value(input::Directional::UpForward, tag("UF")),
        value(input::Directional::UpBackward, tag("UB")),
        value(input::Directional::DownForward, tag("DF")),
        value(input::Directional::DownBackward, tag("DB")),
        value(input::Directional::Up, tag("U")),
        value(input::Directional::Forward, tag("F")),
        value(input::Directional::Down, tag("D")),
        value(input::Directional::Backward, tag("B")),
    ))
    (input)
}

#[cfg(test)]
mod test {
    use super::*;
    use input::*;

    #[test]
    fn test_parse_button() {
        assert_eq!(IResult::Ok(("", Button::C)), parse_button_symbol("c"));
        assert_eq!(IResult::Ok(("", Button::X)), parse_button_symbol("x"));
    }

    #[test]
    fn test_parse_direction() {
        assert_eq!(IResult::Ok(("", Directional::Down)), parse_directional_input("D"));
        assert_eq!(IResult::Ok(("", Directional::DownForward)), parse_directional_input("DF"));
        assert_eq!(IResult::Ok(("", Directional::UpBackward)), parse_directional_input("UB"));
    }

    #[test]
    fn test_parse_state() {
        assert_eq!(IResult::Ok(("", CommandInputState::from(InputSymbol::Buttons(vec![Button::X])))), parse_input_state("x"));
        assert_eq!(IResult::Ok(("", CommandInputState::from(InputSymbol::Buttons(vec![Button::A])))), parse_input_state("a"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: None }, .. CommandInputState::from(InputSymbol::Direction(Directional::Down)) })), parse_input_state("~D"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: None }, .. CommandInputState::from(InputSymbol::Direction(Directional::DownForward)) })), parse_input_state("~DF"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: Some(14) }, .. CommandInputState::from(InputSymbol::Direction(Directional::DownForward)) })), parse_input_state("~14DF"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: Some(7) }, .. CommandInputState::from(InputSymbol::Direction(Directional::Backward)) })), parse_input_state("~7B"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: Some(7) }, strict: true, .. CommandInputState::from(InputSymbol::Direction(Directional::Backward)) })), parse_input_state(">~7B"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: Some(7) }, strict: true, .. CommandInputState::from(InputSymbol::Buttons(vec![Button::X])) })), parse_input_state(">~7x"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::Release { time: Some(7) }, strict: true, .. CommandInputState::from(InputSymbol::Buttons(vec![Button::X, Button::B])) })), parse_input_state(">~7x+b"));
        assert_eq!(IResult::Ok(("", CommandInputState { modifier: InputModifier::HoldDown, strict: false, .. CommandInputState::from(InputSymbol::Buttons(vec![Button::X, Button::B, Button::A, Button::C])) })), parse_input_state("/x+b+a+c"));
    }

    #[test]
    fn parser_test_successive_states() {
        let parsing_result = parse_command_input("~D, DF,/F, x + c + z,~10F");
        assert!(parsing_result.is_ok());
        let command_states = parsing_result.unwrap();
        assert_eq!(
            vec![
                CommandInputState { modifier: InputModifier::Release { time: None }, .. CommandInputState::from(InputSymbol::Direction(Directional::Down)) },
                CommandInputState::from(InputSymbol::Direction(Directional::DownForward)),
                CommandInputState { strict: true, modifier: InputModifier::Release { time: None }, .. CommandInputState::from(InputSymbol::Direction(Directional::DownForward)) },
                CommandInputState { strict: true, modifier: InputModifier::HoldDown, .. CommandInputState::from(InputSymbol::Direction(Directional::Forward)) },
                CommandInputState::from(InputSymbol::Buttons(vec![Button::X, Button::C, Button::Z])),
                CommandInputState { modifier: InputModifier::Release { time: Some(10) }, .. CommandInputState::from(InputSymbol::Direction(Directional::Forward)) },
            ],
            command_states
        );
    }
}
