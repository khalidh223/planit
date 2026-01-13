use super::{arg_extractor::*, arg_matcher::*, arg_parser::*, args::*};
use crate::arg::arg_emitter::{
    ArgEmitter, CardArgEmitter, EventArgEmitter, NoRefEmitContext, SaveEmitContext, TaskArgEmitter,
};
use crate::arg::arg_parse_strategy::{ArgParseStrategy, CommandArgParser, ManArgParser};
use crate::core::aliases::{IdLookup, TokenList};
use crate::core::types::{
    Bool, BoolFormat, Date, DateFormat, DayOfWeek, EntityType, Flag, TimeFormat, TimeRange,
};
use crate::core::{models::Card, models::Event, models::Task, types::CardColor};
use crate::errors::Error;
use crate::extensions::enums::valid_csv;

// ---------- args.rs ----------
#[test]
fn token_stream_walks_tokens() {
    let raw = vec!["one".to_string(), "two".to_string()];
    let mut ts = TokenStream::new(&raw);
    assert!(!ts.eof());
    assert_eq!(ts.peek().unwrap(), "one");
    assert_eq!(ts.next().unwrap(), "one");
    assert_eq!(ts.peek().unwrap(), "two");
    assert_eq!(ts.next().unwrap(), "two");
    assert!(ts.eof());
}

#[test]
fn name_arg_requires_quotes_and_strips_them() {
    let raw = "\"My Task\"";
    assert!(NameArg::accepts(raw));
    let parsed = NameArg::new(raw).unwrap();
    match parsed {
        Arg::Name(s) => assert_eq!(s, "My Task"),
        _ => panic!("expected name arg"),
    }
    assert!(!NameArg::accepts("NoQuotes"));
    assert!(NameArg::new("NoQuotes").is_err());
}

#[test]
fn days_of_week_accepts_list_and_parses() {
    assert!(DaysOfWeekArg::starts_sequence("mon,"));
    assert!(DaysOfWeekArg::starts_sequence("fri"));
    assert!(!DaysOfWeekArg::starts_sequence("xyz"));

    let parsed = DaysOfWeekArg::new("mon, tue").unwrap();
    match parsed {
        Arg::DaysOfWeek(days) => assert_eq!(days, vec![DayOfWeek::Mon, DayOfWeek::Tue]),
        _ => panic!("expected days of week"),
    }
    assert!(DaysOfWeekArg::new("foo").is_err());
}

#[test]
fn card_color_id_validates_prefix_and_number() {
    assert!(CardColorIdArg::accepts("+C12"));
    assert!(!CardColorIdArg::accepts("C12"));
    assert!(!CardColorIdArg::accepts("+C"));

    match CardColorIdArg::new("+C7").unwrap() {
        Arg::CardColorId(v) => assert_eq!(v, 7),
        _ => panic!("expected card color id"),
    }
}

#[test]
fn factories_parse_single_and_multi_token_args() {
    let mut ts = TokenStream::new(&vec!["\"Hello".into(), "World\"".into()]);
    let name_factory = MultiTokenFactory::<NameArg>::new();
    assert!(name_factory.can_start("\"Hello"));
    let arg = name_factory.parse(&mut ts).unwrap();
    assert!(matches!(arg, Arg::Name(ref s) if s == "Hello World"));

    let mut ts = TokenStream::new(&vec!["42".into()]);
    let int_factory = SingleTokenFactory::<IntArg>::new();
    assert!(int_factory.can_start("42"));
    let arg = int_factory.parse(&mut ts).unwrap();
    assert!(matches!(arg, Arg::Int(42)));
}

#[test]
fn date_and_time_args_parse_valid_strings() {
    match DateArg::new("2025-01-05").unwrap() {
        Arg::Date(d) => assert_eq!(d.0.to_string(), "2025-01-05"),
        _ => panic!("expected date"),
    }

    match TimeRangeArg::new("8AM-9AM").unwrap() {
        Arg::TimeRange(tr) => {
            assert!(tr.start < tr.end);
        }
        _ => panic!("expected time range"),
    }
}

#[test]
fn entity_type_arg_parses_and_validates() {
    assert!(EntityTypeArg::accepts("task"));
    match EntityTypeArg::new("event").unwrap() {
        Arg::EntityType(t) => assert_eq!(t, EntityType::Event),
        _ => panic!("expected entity type"),
    }
}

// ---------- arg_parser.rs ----------
#[test]
fn parses_mixed_argument_sequence() {
    let parser = ArgParser::new();
    let raw = vec![
        "\"Write".to_string(),
        "docs\"".to_string(),
        "3".to_string(),
        "+C2".to_string(),
        "@".to_string(),
        "2025-01-02".to_string(),
    ];

    let args = parser.parse(&raw).expect("parse should succeed");
    assert!(matches!(args[0], Arg::Name(ref s) if s == "Write docs"));
    assert!(matches!(args[1], Arg::Int(3)));
    assert!(matches!(args[2], Arg::CardColorId(2)));
    assert!(matches!(args[3], Arg::AtSymbol));
    assert!(matches!(args[4], Arg::Date(_)));
}

#[test]
fn parses_days_and_time_range() {
    let parser = ArgParser::new();
    let raw = vec!["mon,".into(), "tue".into(), "8AM-9AM".into()];

    let args = parser.parse(&raw).expect("parse should succeed");
    assert_eq!(args.len(), 2);
    match &args[0] {
        Arg::DaysOfWeek(days) => assert_eq!(days, &vec![DayOfWeek::Mon, DayOfWeek::Tue]),
        _ => panic!("expected days of week"),
    }
    assert!(matches!(args[1], Arg::TimeRange(_)));
}

#[test]
fn errors_on_unrecognized_argument() {
    let parser = ArgParser::new();
    let err = parser.parse(&["???".into()]).unwrap_err();
    match err {
        Error::Parse(msg) => assert!(msg.contains("Unrecognized argument")),
        other => panic!("expected parse error, got {other:?}"),
    }
}

// ---------- arg_matcher.rs ----------
#[test]
fn matches_variant_checks_correct_type() {
    let name = Arg::Name("test".into());
    let int = Arg::Int(5);
    assert!(NameArg::matches_variant(&name));
    assert!(!NameArg::matches_variant(&int));
    assert!(IntArg::matches_variant(&int));
    assert!(!IntArg::matches_variant(&name));
}

#[test]
fn expected_error_messages_are_descriptive() {
    let err = TimeRangeArg::expected_error(&Arg::Int(3));
    match err {
        Error::Parse(msg) => {
            assert!(msg.contains("time range"));
            assert!(msg.contains("Int"));
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn bool_arg_matcher_formats_expected_error() {
    let good = Arg::Bool(Bool(true));
    assert!(BoolArg::matches_variant(&good));

    let provided = Arg::Int(3);
    let err = BoolArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a boolean, got {:?}. Valid booleans: {}",
                provided,
                valid_csv::<BoolFormat>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn flag_arg_matcher_formats_expected_error() {
    let good = Arg::Flag(Flag::Help);
    assert!(FlagArg::matches_variant(&good));

    let provided = Arg::Int(7);
    let err = FlagArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a flag, got {:?}. Valid flags: {}",
                provided,
                valid_csv::<Flag>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn card_color_arg_matcher_formats_expected_error() {
    let good = Arg::CardColor(CardColor::Green);
    assert!(CardColorArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = CardColorArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a card color, got {:?}. Valid colors: {}",
                provided,
                valid_csv::<CardColor>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn card_color_id_arg_matcher_formats_expected_error() {
    let good = Arg::CardColorId(2);
    assert!(CardColorIdArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = CardColorIdArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a card color id in the format '+C<integer>', got {:?}.",
                provided
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn at_symbol_arg_matcher_formats_expected_error() {
    let good = Arg::AtSymbol;
    assert!(AtSymbolArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = AtSymbolArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!("Expected an @ symbol, got {:?}.", provided);
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn days_of_week_arg_matcher_formats_expected_error() {
    let good = Arg::DaysOfWeek(vec![DayOfWeek::Mon]);
    assert!(DaysOfWeekArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = DaysOfWeekArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a comma separated list of days in the week, got {:?}. Valid days: {}",
                provided,
                valid_csv::<DayOfWeek>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn date_arg_matcher_formats_expected_error() {
    let good = Arg::Date(Date::try_from_str("2025-01-02").unwrap());
    assert!(DateArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = DateArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a valid date, got {:?}. Valid date formats: {}",
                provided,
                valid_csv::<DateFormat>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn entity_type_arg_matcher_formats_expected_error() {
    let good = Arg::EntityType(EntityType::Task);
    assert!(EntityTypeArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = EntityTypeArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected an entity type, got {:?}. Valid entity types {}",
                provided,
                valid_csv::<EntityType>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn time_range_arg_matcher_formats_expected_error() {
    let good = Arg::TimeRange(TimeRange::try_from_str("8AM-9AM").unwrap());
    assert!(TimeRangeArg::matches_variant(&good));

    let provided = Arg::Int(1);
    let err = TimeRangeArg::expected_error(&provided);
    match err {
        Error::Parse(msg) => {
            let expected = format!(
                "Expected a time range in the format <start>-<end>, got {:?}. Supported time formats: {}",
                provided,
                valid_csv::<TimeFormat>()
            );
            assert_eq!(msg, expected);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

// ---------- arg_extractor.rs ----------
#[test]
fn extract_at_returns_expected_variants() {
    let date = Date::try_from_str("2025-01-01").unwrap();
    let tr = TimeRange::try_from_str("8AM-9AM").unwrap();
    let args = vec![
        Arg::Name("Alpha".into()),
        Arg::Int(5),
        Arg::Bool(Bool(true)),
        Arg::Date(date.clone()),
        Arg::TimeRange(tr.clone()),
        Arg::Flag(Flag::Help),
        Arg::CardColorId(7),
    ];

    assert_eq!(extract_at::<NameArg>(&args, 0), "Alpha");
    assert_eq!(extract_at::<IntArg>(&args, 1), 5);
    assert_eq!(extract_at::<BoolArg>(&args, 2), Bool(true));
    assert_eq!(extract_at::<DateArg>(&args, 3), &date);
    assert_eq!(extract_at::<TimeRangeArg>(&args, 4), &tr);
    assert_eq!(extract_at::<FlagArg>(&args, 5), Flag::Help);
    assert_eq!(extract_at::<CardColorIdArg>(&args, 6), 7);
}

#[test]
fn extract_at_returns_entity_type() {
    let args = vec![Arg::EntityType(EntityType::Event)];
    let value = extract_at::<EntityTypeArg>(&args, 0);
    assert_eq!(value, EntityType::Event);
}

#[test]
fn extract_at_returns_card_color() {
    let args = vec![Arg::CardColor(CardColor::Red)];
    let value = extract_at::<CardColorArg>(&args, 0);
    assert_eq!(value, CardColor::Red);
}

#[test]
fn extract_at_returns_days_of_week() {
    let args = vec![Arg::DaysOfWeek(vec![DayOfWeek::Fri])];
    let value = extract_at::<DaysOfWeekArg>(&args, 0);
    assert_eq!(value, &vec![DayOfWeek::Fri]);
}

#[test]
fn extract_at_accepts_at_symbol() {
    let args = vec![Arg::AtSymbol];
    let value = extract_at::<AtSymbolArg>(&args, 0);
    assert_eq!(value, ());
}

#[test]
fn try_extract_returns_none_on_mismatch() {
    let arg = Arg::Int(10);
    assert!(NameArg::try_extract(&arg).is_none());
    assert!(BoolArg::try_extract(&Arg::Name("x".into())).is_none());
}

#[test]
#[should_panic(expected = "Expected an integer")]
fn extract_at_panics_on_incorrect_variant() {
    let args = vec![Arg::Name("oops".into())];
    let _ = extract_at::<IntArg>(&args, 0);
}

// ---------- arg_emitter.rs ----------

fn assert_arg_strings(args: &[Arg], expected: &[&str]) {
    let rendered: TokenList = args.iter().map(|a| a.to_string()).collect();
    assert_eq!(rendered, expected);
}

#[test]
fn card_arg_emitter_emits_name_and_color() {
    let emitter = CardArgEmitter::new();
    let card = Card::new("hello", CardColor::Red);
    let ctx = NoRefEmitContext;

    let args = emitter.with_entity(&card, &ctx).unwrap();
    assert_arg_strings(&args, &["\"hello\"", "RED"]);
}

#[test]
fn task_arg_emitter_maps_card_and_orders_args() {
    let emitter = TaskArgEmitter::new();
    let task = Task::new(
        "work",
        3.5,
        Some(7),
        Date::try_from_str("2025-02-01").unwrap(),
    );
    let mut card_map = IdLookup::new();
    card_map.insert(7, 2);
    let ctx = SaveEmitContext {
        id_lookup: &card_map,
    };

    let args = emitter.with_entity(&task, &ctx).unwrap();
    assert_arg_strings(&args, &["\"work\"", "4", "+C2", "@", "2025-02-01"]);
}

#[test]
fn task_arg_emitter_errors_when_card_unmapped() {
    let emitter = TaskArgEmitter::new();
    let task = Task::new(
        "work",
        1.0,
        Some(9),
        Date::try_from_str("2025-02-01").unwrap(),
    );
    let map = IdLookup::new();
    let ctx = SaveEmitContext { id_lookup: &map };

    let err = emitter.with_entity(&task, &ctx).unwrap_err();
    match err {
        Error::Parse(msg) => assert!(msg.contains("Reference to missing card id 9")),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn event_arg_emitter_emits_days_when_present() {
    let emitter = EventArgEmitter::new();
    let event = Event::new(
        true,
        "party",
        Some(3),
        vec![DayOfWeek::Mon, DayOfWeek::Wed],
        TimeRange::try_from_str("8AM-10AM").unwrap(),
    );
    let mut map = IdLookup::new();
    map.insert(3, 1);
    let ctx = SaveEmitContext { id_lookup: &map };

    let args = emitter.with_entity(&event, &ctx).unwrap();
    assert_arg_strings(
        &args,
        &[
            "True",
            "\"party\"",
            "+C1",
            "@",
            "MON, WED",
            "8:00AM-10:00AM",
        ],
    );
}

#[test]
fn event_arg_emitter_omits_days_when_empty() {
    let emitter = EventArgEmitter::new();
    let event = Event::new(
        false,
        "solo",
        None,
        Vec::new(),
        TimeRange::try_from_str("1PM-2PM").unwrap(),
    );
    let map = IdLookup::new();
    let ctx = SaveEmitContext { id_lookup: &map };

    let args = emitter.with_entity(&event, &ctx).unwrap();
    assert_arg_strings(&args, &["False", "\"solo\"", "@", "1:00PM-2:00PM"]);
}

// ---------- arg_parse_strategy.rs ----------

#[test]
fn command_arg_parser_uses_default_parser() {
    let parser = CommandArgParser::new();
    let raw = vec![
        "\"Task\"".to_string(),
        "1".to_string(),
        "@".to_string(),
        "2099-01-01".to_string(),
    ];
    let args = parser.parse("task", &raw).unwrap();
    match &args[0] {
        Arg::Name(name) => assert_eq!(name, "Task"),
        other => panic!("expected name arg, got {other:?}"),
    }
}

#[test]
fn command_arg_parser_uses_manual_parser() {
    let parser = CommandArgParser::new();
    let raw = vec!["config".to_string()];
    let args = parser.parse("man", &raw).unwrap();
    match &args[..] {
        [Arg::Name(name)] => assert_eq!(name, "config"),
        other => panic!("expected single name arg, got {other:?}"),
    }
}

#[test]
fn command_arg_parser_joins_multiple_tokens_for_manual() {
    let parser = CommandArgParser::new();
    let raw = vec!["\"schedule".to_string(), "start\"".to_string()];
    let args = parser.parse("man", &raw).unwrap();
    match &args[..] {
        [Arg::Name(name)] => assert_eq!(name, "schedule start"),
        other => panic!("expected single name arg, got {other:?}"),
    }
}

#[test]
fn man_arg_parser_returns_empty_when_no_args() {
    let parser = ManArgParser;
    let args = parser.parse(&[]).unwrap();
    assert!(args.is_empty());
}

#[test]
fn man_arg_parser_returns_name_for_unquoted_topic() {
    let parser = ManArgParser;
    let args = parser.parse(&["config".to_string()]).unwrap();
    match &args[..] {
        [Arg::Name(name)] => assert_eq!(name, "config"),
        other => panic!("expected single name arg, got {other:?}"),
    }
}

#[test]
fn man_arg_parser_strips_wrapping_quotes() {
    let parser = ManArgParser;
    let args = parser.parse(&["\"schedule\"".to_string()]).unwrap();
    match &args[..] {
        [Arg::Name(name)] => assert_eq!(name, "schedule"),
        other => panic!("expected single name arg, got {other:?}"),
    }
}

#[test]
fn man_arg_parser_preserves_joined_phrase() {
    let parser = ManArgParser;
    let args = parser
        .parse(&["\"schedule".to_string(), "start\"".to_string()])
        .unwrap();
    match &args[..] {
        [Arg::Name(name)] => assert_eq!(name, "schedule start"),
        other => panic!("expected single name arg, got {other:?}"),
    }
}
