use crate::game::input;
use nom::{IResult, multi::separated_list1, sequence::{preceded, tuple}, character::complete::{space0, digit1}, combinator::{map, value, opt, complete}, bytes::complete::tag, branch::alt, Parser, error::Error, Finish};
use std::str;
use super::{CommandInputState, ModifiedInput};

pub fn parse_command_input(command_string: &str) -> Result<Vec<CommandInputState>, Error<&str>> {
    let (_, mut command_states) = complete(parse_list)(command_string).finish()?;
    log::trace!("Parsed command string \"{command_string}\": {command_states:?}");
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
                        let mut inserted_directional = CommandInputState::default();
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
            parse_symbols,
        )),
        |(strict, mut partial_state)| {
            partial_state.strict = strict;
            partial_state
        }
    )
    (input)
}

fn parse_symbols(input: &str) -> IResult<&str, CommandInputState> {
    map(
        separated_list1(tag("+"), parse_modified_symbol),
        |partial_states| {
            partial_states.into_iter().fold(CommandInputState::default(), |mut input_state, partial_state| {
                input_state.button_presses.extend(partial_state.button_presses.into_iter());
                input_state.directional = partial_state.directional.or(input_state.directional);
                input_state
            })
        }
    )
    (input)
}

fn parse_modified_symbol(input: &str) -> IResult<&str, CommandInputState>
{
    alt((
        // held down button
        parse_held_down_button,
        // released button
        parse_released_button,
        // held down direction
        parse_held_down_direction,
        // released direction
        parse_released_direction,
        // normal direction
        parse_normal_direction,
        // normal button
        parse_normal_button,
    ))
    (input)
}

fn command_input_state_parser<P, F, E, I, O>(p: P, f: F) -> impl FnMut(I) -> IResult<I, CommandInputState, E>
    where
        P: Parser<I, O, E>,
        E: nom::error::ParseError<I>,
        I: nom::InputTake + nom::InputLength,
        F: Fn(&mut CommandInputState, O) -> (),
{
    map(p,
        move |parser_output| {
            let mut input_state = CommandInputState::default();
            f(&mut input_state, parser_output);
            input_state
        }
    )
}

fn parse_normal_button(input: &str) -> IResult<&str, CommandInputState>
{
    command_input_state_parser(
        parse_button_symbol,
        |input_state, button_symbol| input_state.button_presses.push(ModifiedInput::Normal(button_symbol))
    )
    (input)
}

fn parse_held_down_button(input: &str) -> IResult<&str, CommandInputState>
{
    command_input_state_parser(
        preceded(tag("/"), tuple((map(opt(tag("$")), |o| o.is_some()), parse_button_symbol))),
        // TODO account for partial push
        |input_state, (partial_state, button_symbol)| input_state.button_presses.push(ModifiedInput::HoldDown(button_symbol))
    )
    (input)
}

fn release_modifier(input: &str) -> IResult<&str, Option<u16>>
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

fn parse_released_button(input: &str) -> IResult<&str, CommandInputState>
{
    command_input_state_parser(
        tuple((release_modifier, parse_button_symbol)),
        |input_state, (release_time, button_symbol)| input_state.button_presses.push(ModifiedInput::Release(button_symbol, release_time))
    )
    (input)
}

fn parse_normal_direction(input: &str) -> IResult<&str, CommandInputState>
{
    command_input_state_parser(
        parse_directional_input,
        |input_state, directional: input::PartialDirectional| input_state.directional = Some(ModifiedInput::Normal(directional)),
    )
    (input)
}

fn parse_held_down_direction(input: &str) -> IResult<&str, CommandInputState>
{
    command_input_state_parser(
        preceded(tag("/"), parse_directional_input),
        |input_state, directional: input::PartialDirectional| input_state.directional = Some(ModifiedInput::HoldDown(directional)),
    )
    (input)
}

fn parse_released_direction(input: &str) -> IResult<&str, CommandInputState>
{
    command_input_state_parser(
        tuple((release_modifier, parse_directional_input)),
        |input_state, (release_time, directional)| input_state.directional = Some(ModifiedInput::Release(directional, release_time)),
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

fn parse_directional_input(input: &str) -> IResult<&str, input::PartialDirectional>
{
    alt((
        value(input::PartialDirectional::Vertical(input::DirectionState::Plus), tag("$U")),
        value(input::PartialDirectional::Vertical(input::DirectionState::Minus), tag("$D")),
        value(input::PartialDirectional::Horizontal(input::DirectionState::Plus), tag("$F")),
        value(input::PartialDirectional::Horizontal(input::DirectionState::Minus), tag("$B")),
        complete(alt((
            value(input::PartialDirectional::FullDirection(input::Directional::UpForward), tag("UF")),
            value(input::PartialDirectional::FullDirection(input::Directional::UpBackward), tag("UB")),
            value(input::PartialDirectional::FullDirection(input::Directional::DownForward), tag("DF")),
            value(input::PartialDirectional::FullDirection(input::Directional::DownBackward), tag("DB")),
        ))),
        value(input::PartialDirectional::FullDirection(input::Directional::Up), tag("U")),
        value(input::PartialDirectional::FullDirection(input::Directional::Forward), tag("F")),
        value(input::PartialDirectional::FullDirection(input::Directional::Down), tag("D")),
        value(input::PartialDirectional::FullDirection(input::Directional::Backward), tag("B")),
    ))
    (input)
}

#[cfg(test)]
mod test {
    use super::*;
    use input::*;
    // testing the parser
    // first test only bits of parsing expressions
    #[test]
    fn parser_test_bits() {
        assert_eq!(IResult::Ok(("", Button::C)), parse_button_symbol("c"));
        assert_eq!(IResult::Ok(("", Button::X)), parse_button_symbol("x"));
        assert_eq!(IResult::Ok(("", PartialDirectional::FullDirection(Directional::Down))), parse_directional_input("D"));
        assert_eq!(IResult::Ok(("", PartialDirectional::FullDirection(Directional::DownForward))), parse_directional_input("DF"));
        assert_eq!(IResult::Ok(("",  CommandInputState { directional: Some(ModifiedInput::Release(PartialDirectional::FullDirection(Directional::Down), None)), .. Default::default() })), parse_modified_symbol("~D"));
        assert_eq!(IResult::Ok(("",  CommandInputState { directional: Some(ModifiedInput::Release(PartialDirectional::FullDirection(Directional::UpBackward), None)), .. Default::default() })), parse_modified_symbol("~UB"));
        assert_eq!(IResult::Ok(("",  CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], .. Default::default() })), parse_normal_button("x"));
        assert_eq!(IResult::Ok(("",  CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], .. Default::default() })), parse_modified_symbol("x"));
        assert_eq!(IResult::Ok(("",  CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], .. Default::default() })), parse_symbols("x"));
        assert_eq!(IResult::Ok(("",  CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], strict:true, .. Default::default() })), parse_input_state(">x"));
        assert_eq!(IResult::Ok(("",  vec![CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], strict: true, .. Default::default() }])), parse_list(">x"));
    }
    #[test]
    fn parser_test_x() {
        assert_eq!(Result::Ok(vec![CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], .. Default::default() }]), parse_command_input("x"));
        assert_eq!(Result::Ok(vec![CommandInputState { button_presses: vec![ModifiedInput::Normal(input::Button::X)], .. Default::default() }]), parse_command_input(" x"));
    }
    // quarter circle forward + x
    #[test]
    fn parser_test_qcf_x() {
        let parsing_result = parse_command_input("~D, DF, F, x");
        assert!(parsing_result.is_ok());
        let command_states = parsing_result.unwrap();
        assert_eq!(
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
            ],
            command_states
        );
    }
}
