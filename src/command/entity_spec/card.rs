use crate::arg::args::{Arg, CardColorArg, NameArg};
use crate::command::entity_spec::common::{entity_slot, id_slot};
use crate::command::entity_spec::core::{
    ArgPattern, ArgSchema, ArgSlot, ArgValidator, ColumnIndexer, EntityBuilder, EntitySpec,
    PatternIdExt,
};
use crate::core::context::AppContext;
use crate::core::models::Card;
use crate::core::types::{EntityActionType, EntityType};
use crate::errors::{Error, Result};
use std::fmt;

pub struct CardArgSchema;
impl CardArgSchema {
    fn pattern_base() -> ArgPattern {
        vec![
            ArgSlot::is_of_arg_type::<NameArg>(),
            ArgSlot::is_of_arg_type::<CardColorArg>(),
        ]
    }

    fn pattern_entity_id() -> ArgPattern {
        vec![entity_slot(EntityType::Card), id_slot()]
    }

    fn pattern_entity_first() -> ArgPattern {
        let mut v = Self::pattern_entity_id();
        v.extend(Self::pattern_base());
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardPat {
    Base,
    EntityFirst,
    EntityId,
}

impl CardPat {
    const fn usage(self) -> &'static str {
        match self {
            CardPat::Base => {
                r#"card "<name>" <color>
Required:
  name  - (string)    Name of card, wrapped in single or double quotes
  color - (CardColor) Valid card color. Run 'colors -h' to see valid card colors."#
            }

            CardPat::EntityFirst => {
                r#"card <id> "<name>" <color>
Required:
  id    - (int)       id of card
  name  - (string)    Name of card, wrapped in single or double quotes
  color - (CardColor) Valid card color. Run 'colors -h' to see valid card colors"#
            }

            CardPat::EntityId => {
                r#"card <id>
Required:
  id    - (int)       id of card"#
            }
        }
    }
}

impl fmt::Display for CardPat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.usage())
    }
}

impl PatternIdExt for CardPat {
    fn pattern(&self) -> ArgPattern {
        match self {
            CardPat::Base => CardArgSchema::pattern_base(),
            CardPat::EntityFirst => CardArgSchema::pattern_entity_first(),
            CardPat::EntityId => CardArgSchema::pattern_entity_id(),
        }
    }
}

impl ArgSchema for CardArgSchema {
    type PatternId = CardPat;

    fn patterns_for(&self, action: EntityActionType) -> Vec<CardPat> {
        match action {
            EntityActionType::Add => vec![CardPat::Base],
            EntityActionType::Modify => vec![CardPat::EntityFirst],
            EntityActionType::Delete => vec![CardPat::EntityId],
        }
    }
}

pub struct CardArgValidator;
impl ArgValidator for CardArgValidator {
    type PatternId = CardPat;

    fn validate(
        &self,
        _args: &[Arg],
        _action: EntityActionType,
        _pat_id: Self::PatternId,
    ) -> Result<()> {
        Ok(())
    }
}

pub struct CardBuilder;

impl EntityBuilder<Card> for CardBuilder {
    type PatternId = CardPat;

    fn create(&self, args: &[Arg], pat_id: CardPat) -> Result<Card> {
        match pat_id {
            CardPat::Base => {
                let pattern = pat_id.pattern();
                let mut ix = ColumnIndexer::new(args, &pattern);
                Ok(Card::new(
                    ix.next::<NameArg>().clone(),
                    ix.next::<CardColorArg>().clone(),
                ))
            }
            _ => Err(Error::Parse(
                "No valid ADD pattern matched for card.".into(),
            )),
        }
    }

    fn modify<'a>(
        &self,
        existing: &'a mut Card,
        args: &[Arg],
        pat_id: CardPat,
    ) -> Result<&'a Card> {
        match pat_id {
            CardPat::EntityFirst => {
                let pattern = pat_id.pattern();
                let mut ix = ColumnIndexer::new(args, &pattern);
                existing.modify(
                    ix.advance_times(2).next::<NameArg>().clone(),
                    ix.next::<CardColorArg>().clone(),
                );
                Ok(&*existing)
            }
            _ => Err(Error::Parse(
                "No valid MODIFY pattern matched for card.".into(),
            )),
        }
    }
}

pub struct CardSpec {
    schema: CardArgSchema,
    validator: CardArgValidator,
    builder: CardBuilder,
}

impl CardSpec {
    pub fn new() -> Self {
        Self {
            schema: CardArgSchema,
            validator: CardArgValidator,
            builder: CardBuilder,
        }
    }
}

impl EntitySpec<Card> for CardSpec {
    type PatternId = CardPat;

    fn arg_schema(&self) -> &dyn ArgSchema<PatternId = CardPat> {
        &self.schema
    }
    fn arg_validator(&self) -> &dyn ArgValidator<PatternId = CardPat> {
        &self.validator
    }
    fn entity_builder(&self) -> &dyn EntityBuilder<Card, PatternId = CardPat> {
        &self.builder
    }

    fn get_mut<'a>(&self, ctx: &'a mut AppContext, id: i32) -> Result<&'a mut Card> {
        ctx.cards.get_mut(id)
    }
}
