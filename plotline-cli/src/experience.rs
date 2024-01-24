use crate::{Error, Result};
use clap::{Args, Subcommand};
use plotline::{
    entity::{application::EntityRepository, Entity},
    event::{application::EventRepository, Event},
    experience::{
        application::{ConstraintFactory, ExperienceApplication, ExperienceRepository},
        Experience,
    },
    id::{Id, Identifiable, Result as IdResult},
};
use prettytable::Table;
use std::fmt::Display;

#[derive(Args)]
struct ProfileSetArgs {
    /// The name of the field.
    key: String,
    /// The field's value.
    value: Option<String>,
}

#[derive(Subcommand)]
enum ProfileCommand {
    /// Set a new value in a profile.
    Set(ProfileSetArgs),
    /// List all profiles.
    #[clap(alias = "ls")]
    List,
    /// Remove a profile.
    #[clap(alias = "rm")]
    Remove,
}

#[derive(Args)]
struct ProfileArgs {
    /// The id of the profile's entity.
    entity: Option<String>,
    /// The action to perform.
    #[command(subcommand)]
    command: Option<ProfileCommand>,
}

#[derive(Args)]
struct ExperienceSaveArgs {
    /// Mark the experience as terminal.
    #[clap(short, long)]
    terminal: bool,
}

#[derive(Subcommand)]
#[clap(subcommand_negates_reqs = true, subcommand_precedence_over_arg = true)]
enum ExperienceSubCommand {
    /// Save an experience.
    Save(ExperienceSaveArgs),
    /// List all experiences.
    #[command(alias("ls"))]
    List,
    /// Manage profiles.
    Profile(ProfileArgs),
}

#[derive(Args)]
pub struct ExperienceIdArgs {
    /// The id of the entity involved in the experience.
    entity: String,
    /// The id of the event causing the experience.
    event: String,
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct ExperienceCommand {
    /// The id of the experience.
    #[clap(flatten)]
    experience: Option<ExperienceIdArgs>,
    /// The action to perform.
    #[command(subcommand)]
    command: Option<ExperienceSubCommand>,
}

pub struct ExperienceCli<ExperienceRepo, EntityRepo, EventRepo, CnstFactory> {
    pub experience_app: ExperienceApplication<ExperienceRepo, EntityRepo, EventRepo, CnstFactory>,
}

impl<ExperienceRepo, EntityRepo, EventRepo, CnstFactory>
    ExperienceCli<ExperienceRepo, EntityRepo, EventRepo, CnstFactory>
where
    ExperienceRepo: 'static + ExperienceRepository<Interval = EventRepo::Interval> + Sync + Send,
    EntityRepo: 'static + EntityRepository + Sync + Send,
    EventRepo: 'static + EventRepository + Sync + Send,
    CnstFactory: 'static + ConstraintFactory<EventRepo::Interval> + Sync + Send,
{
    /// Given an [ExperienceCommand], executes the corresponding logic.
    pub fn execute(&self, experience_cmd: ExperienceCommand) -> Result {
        let experience_id = experience_cmd
            .experience
            .map(|experience| -> IdResult<_> {
                Ok((experience.entity.try_into()?, experience.event.try_into()?))
            })
            .transpose()?;

        if let Some(command) = experience_cmd.command {
            return self.execute_subcommand(command, experience_id);
        }

        Ok(())
    }

    fn execute_subcommand(
        &self,
        subcommand: ExperienceSubCommand,
        experience: Option<<Experience<EventRepo::Interval> as Identifiable>::Id>,
    ) -> Result {
        match subcommand {
            ExperienceSubCommand::Save(args) => {
                let (entity_id, event_id) =
                    experience.ok_or(Error::MissingArgument("experience id"))?;

                self.experience_app
                    .save_experience(entity_id, event_id)
                    .with_after(args.terminal.then_some(Vec::default()));

                println!("{} {}", entity_id, event_id);
            }
            ExperienceSubCommand::List => {
                let experiences = self.experience_app.filter_experiences().execute()?;
                print!("{}", ManyExperiencesFmt::new(&experiences));
            }
            ExperienceSubCommand::Profile(args) => self.execute_profile_command(
                experience.ok_or(Error::MissingArgument("experience id"))?,
                args.entity.map(TryInto::try_into).transpose()?,
                args.command,
            ),
        }

        Ok(())
    }

    fn execute_profile_command(
        &self,
        _experience: (Id<Entity>, Id<Event<EventRepo::Interval>>),
        _entity: Option<Id<Entity>>,
        _command: Option<ProfileCommand>,
    ) {
        todo!()
    }
}

struct ManyExperiencesFmt<'a, Intv> {
    experiences: &'a [Experience<Intv>],
}

impl<'a, Intv> Display for ManyExperiencesFmt<'a, Intv> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.add_row(row!["ENTITY ID", "EVENT ID"]);
        self.experiences.iter().for_each(|experience| {
            table.add_row(row![&experience.entity, &experience.event]);
        });

        table.fmt(f)
    }
}

impl<'a, Intv> ManyExperiencesFmt<'a, Intv> {
    pub fn new(experiences: &'a [Experience<Intv>]) -> Self {
        Self { experiences }
    }
}
