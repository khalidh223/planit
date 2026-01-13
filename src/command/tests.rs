use super::command_parser::CommandParser;
use super::command_resolver::{
    AddEntityResolver, CommandResolver, EntityActionResolver, GlobalResolver, TypeHelpResolver,
};
use crate::arg::args::Arg;
use crate::command::commands::Command;
use crate::command::manual::{ManualCatalog, ManualTopic};
use crate::core::types::EntityType;
use crate::errors::Error;
use strum::IntoEnumIterator;

fn usage_of(cmd: &dyn Command<'_>) -> String {
    cmd.usage()
}

#[test]
fn command_parser_resolves_add_entity() {
    let parser = CommandParser::new();
    let args: Vec<Arg> = vec![];
    let cmd = parser
        .parse("task", &args)
        .expect("should resolve add task");
    // usage should mention task add patterns
    let usage = cmd.usage();
    assert!(usage.to_lowercase().contains("task"));
}

#[test]
fn command_parser_unknown_command_errors() {
    let parser = CommandParser::new();
    assert!(matches!(
        parser.parse("does-not-exist", &[]),
        Err(Error::UnknownCommand(_))
    ));
}

#[test]
fn entity_action_resolver_requires_entity_type_arg() {
    let resolver = EntityActionResolver;
    assert!(resolver.can_resolve("mod"));
    match resolver.resolve("mod", &[]) {
        Err(Error::Parse(msg)) => assert!(msg.contains("Expected entity type")),
        _ => panic!("expected parse error"),
    }
}

#[test]
fn entity_action_resolver_builds_command_when_args_valid() {
    let resolver = EntityActionResolver;
    let args = vec![Arg::EntityType(EntityType::Task), Arg::Int(1)];
    let cmd = resolver
        .resolve("mod", &args)
        .expect("should resolve modify command");
    let usage = usage_of(cmd.as_ref());
    assert!(usage.to_lowercase().contains("task"));
}

#[test]
fn add_entity_resolver_matches_entity_types() {
    let resolver = AddEntityResolver;
    assert!(resolver.can_resolve("task"));
    assert!(resolver.can_resolve("event"));
    assert!(resolver.can_resolve("card"));
    assert!(!resolver.can_resolve("mod"));
}

#[test]
fn global_resolver_matches_schedule_and_config() {
    let resolver = GlobalResolver;
    assert!(resolver.can_resolve("schedule"));
    assert!(resolver.can_resolve("config"));
    assert!(resolver.can_resolve("man"));

    let schedule_cmd = resolver
        .resolve("schedule", &[])
        .expect("schedule should resolve");
    assert!(schedule_cmd.usage().to_lowercase().contains("schedule"));

    let config_cmd = resolver
        .resolve("config", &[])
        .expect("config should resolve");
    assert!(config_cmd.usage().to_lowercase().contains("config"));

    let man_cmd = resolver.resolve("man", &[]).expect("man should resolve");
    assert!(man_cmd.usage().to_lowercase().contains("man"));
}

#[test]
fn type_help_resolver_matches_known_types() {
    let resolver = TypeHelpResolver;
    for cmd in ["date", "time", "colors"] {
        assert!(resolver.can_resolve(cmd));
        let usage_cmd = resolver
            .resolve(cmd, &[])
            .expect("should resolve type help command");
        let usage = usage_cmd.usage();
        assert!(!usage.is_empty());
    }
    assert!(!resolver.can_resolve("unknown"));
}

#[test]
fn manual_catalog_renders_general_page() {
    let page = ManualCatalog::new().page_for(None).unwrap();
    let output = page.render();
    assert!(output.contains("NAME"));
    assert!(output.contains("planit"));
}

#[test]
fn manual_catalog_errors_on_unknown_topic() {
    let err = ManualCatalog::new().page_for(Some("unknown")).unwrap_err();
    match err {
        Error::Parse(msg) => assert!(msg.contains("Valid topics")),
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn manual_catalog_renders_pages_for_all_topics() {
    let catalog = ManualCatalog::new();
    for topic in ManualTopic::iter() {
        let page = catalog.page_for(Some(topic.as_ref())).unwrap();
        let output = page.render();
        assert!(!output.is_empty());
    }
}

#[test]
fn manual_task_page_includes_modify_and_delete_usage() {
    let page = ManualCatalog::new().page_for(Some("task")).unwrap();
    let output = page.render();
    assert!(output.contains("mod task"));
    assert!(output.contains("del task"));
}

#[test]
fn manual_type_help_page_includes_usage_lines() {
    let page = ManualCatalog::new().page_for(Some("date")).unwrap();
    let output = page.render();
    assert!(output.contains("date"));
    assert!(output.contains("Supported formats"));
}
