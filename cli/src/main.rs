use clap::{Parser, Subcommand};
use flern::model::{CrudRepository, DatabaseError, DbConnection, ModelManager};
use flern::model::entity::{
    Answer,
    AnswerCreate,
    Lesson,
    LessonCreate,
    LessonTask,
    LessonTaskCreate,
    Module,
    ModuleCreate,
    UserEntity,
    UserEntityCreateUpdate,
};
use flern::web::AuthenticatedUser;

#[derive(Parser, Debug)]
#[command(about = "CLI tool for filling the learning DB", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage users
    User {
        #[command(subcommand)]
        action: UserCommands,
    },

    /// Manage modules
    Module {
        #[command(subcommand)]
        action: ModuleCommands,
    },

    /// Manage lessons
    Lesson {
        #[command(subcommand)]
        action: LessonCommands,
    },

    /// Manage tasks
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
}

/// User management
#[derive(Subcommand, Debug)]
pub enum UserCommands {
    Add {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
        #[arg(long, default_value = "user")]
        role: String,
    },
}

/// Module management
#[derive(Subcommand, Debug)]
pub enum ModuleCommands {
    Add {
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: String,
        #[arg(long, default_value_t = 0)]
        order_index: i32,
    },
}

/// Lesson management
#[derive(Subcommand, Debug)]
pub enum LessonCommands {
    Add {
        /// Module title to attach the lesson to
        #[arg(long)]
        module_title: String,
        #[arg(long)]
        title: String,
        /// Path to a Markdown file with lesson content
        #[arg(long)]
        file: String,
        #[arg(long, default_value_t = 0)]
        order_index: i32,
    },
}

/// Task management
#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    Add {
        /// Lesson title to attach the task to
        #[arg(long)]
        lesson_title: String,
        #[arg(long)]
        task_type: String, // validate later
        #[arg(long)]
        question: String,
        #[arg(long)]
        explanation: String,
    },
    AddAnswer {
        /// Task question to attach the answer to
        #[arg(long)]
        task_question: String,
        #[arg(long)]
        answer_text: String,
        #[arg(long, default_value = "")]
        image: String,
        #[arg(long, default_value_t = false)]
        is_correct: bool,
    },
}


#[tokio::main]
async fn main() -> flern::error::AppResult<()> {
    let _ = dotenvy::dotenv();
    let args = Cli::parse();

    let db_con = DbConnection::connect(&std::env::var("DATABASE_URL").unwrap())?;
    let mm = ModelManager::new(db_con);
    let actor = AuthenticatedUser::admin();

    match args.command {
        Commands::User { action } => match action {
            UserCommands::Add { username, password, .. } => {
                let user = UserEntity::create(
                    &mm,
                    &actor,
                    UserEntityCreateUpdate {
                        username,
                        password_hash: flern::auth::hash_password(&password).unwrap(),
                    },
                )
                .await?;
                println!("User created: {:?}", user);
            }
        },

        Commands::Module { action } => match action {
            ModuleCommands::Add { title, description, order_index } => {
                let module = Module::create(
                    &mm,
                    &actor,
                    ModuleCreate {
                        title,
                        description,
                        order_index: Some(order_index),
                    },
                )
                .await?;
                println!("Module created: {:?}", module);
            }
        },

        Commands::Lesson { action } => match action {
            LessonCommands::Add { module_title, title, file, order_index } => {
                let module_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM modules WHERE title = $1")
                    .bind(&module_title)
                    .fetch_one(mm.executor())
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e))?;

                let content = std::fs::read_to_string(file)?;
                let lesson = Lesson::create(
                    &mm,
                    &actor,
                    LessonCreate {
                        module_id,
                        title,
                        content,
                        order_index: Some(order_index),
                    },
                )
                .await?;
                println!("Lesson created: {:?}", lesson);
            }
        },

        Commands::Task { action } => match action {
            TaskCommands::Add { lesson_title, task_type, question, explanation } => {
                let lesson_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM lessons WHERE title = $1")
                    .bind(&lesson_title)
                    .fetch_one(mm.executor())
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e))?;

                let task = LessonTask::create(
                    &mm,
                    &actor,
                    LessonTaskCreate {
                        lesson_id,
                        task_type,
                        question,
                        explanation,
                    },
                )
                .await?;
                println!("Task created: {:?}", task);
            }

            TaskCommands::AddAnswer { task_question, answer_text, image, is_correct } => {
                let task_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM tasks WHERE question = $1")
                    .bind(&task_question)
                    .fetch_one(mm.executor())
                    .await
                    .map_err(|e| DatabaseError::SqlxError(e))?;

                let answer = Answer::create(
                    &mm,
                    &actor,
                    AnswerCreate {
                        task_id,
                        answer_text,
                        image,
                        is_correct: Some(is_correct),
                    },
                )
                .await?;
                println!("Answer created: {:?}", answer);
            }
        },
    }

    Ok(())
}

