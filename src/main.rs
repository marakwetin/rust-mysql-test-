use sqlx::{mysql::MySqlPoolOptions, MySqlPool}; // `Row` import removed
use dotenv::dotenv;
use std::io::{self, Write};
use chrono::{NaiveDateTime, Local, TimeZone}; // `TimeZone` imported for Local.from_local_datetime

// Define a struct to represent our Task
#[derive(Debug, sqlx::FromRow)]
struct Task {
    id: i32, // Corrected to i32 to match MySQL's INT
    description: String,
    completed: bool, // Correctly mapped from MySQL's TINYINT(1)
    created_at: NaiveDateTime,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok(); // Load environment variables from .env file

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Create a connection pool
    let pool = MySqlPoolOptions::new()
        .max_connections(5) // Max 5 connections in the pool
        .connect(&database_url)
        .await?;

    println!("Connected to MySQL database!");

    loop {
        println!("\n--- Task Management CLI ---");
        println!("1. Add Task");
        println!("2. List Tasks");
        println!("3. Mark Task as Completed");
        println!("4. Delete Task");
        println!("5. Exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap(); // Ensure the prompt is displayed

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "1" => add_task(&pool).await?,
            "2" => list_tasks(&pool).await?,
            "3" => mark_task_completed(&pool).await?,
            "4" => delete_task(&pool).await?,
            "5" => {
                println!("Exiting application. Goodbye!");
                break;
            },
            _ => println!("Invalid choice. Please try again."),
        }
    }

    Ok(())
}

async fn add_task(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    print!("Enter task description: ");
    io::stdout().flush().unwrap();

    let mut description = String::new();
    io::stdin().read_line(&mut description).expect("Failed to read line");
    let description = description.trim();

    if description.is_empty() {
        println!("Task description cannot be empty.");
        return Ok(());
    }

    let query = sqlx::query!(
        "INSERT INTO tasks (description) VALUES (?)",
        description
    );

    let result = query.execute(pool).await?;

    if result.rows_affected() > 0 {
        println!("Task '{}' added successfully!", description);
    } else {
        println!("Failed to add task.");
    }
    Ok(())
}

async fn list_tasks(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let tasks: Vec<Task> = sqlx::query_as!(
        Task,
        "SELECT id, description, completed AS 'completed!: bool', created_at FROM tasks ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    if tasks.is_empty() {
        println!("No tasks found.");
    } else {
        println!("\n--- Your Tasks ---");
        for task in tasks {
            let status = if task.completed { "[COMPLETED]" } else { "[PENDING]" };
            
            // FIX: Correctly converting NaiveDateTime from DB to DateTime<Local>
            let created_at_local: chrono::DateTime<Local> = Local.from_local_datetime(&task.created_at)
                .earliest() // Handles potential DST ambiguities by picking the earlier time
                .expect("Failed to convert naive datetime to local datetime"); // Will panic if conversion is impossible (e.g., non-existent time during DST)

            println!("ID: {}, {} Description: '{}' (Created: {})", task.id, status, task.description, created_at_local.format("%Y-%m-%d %H:%M:%S"));
        }
    }
    Ok(())
}

async fn mark_task_completed(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    print!("Enter the ID of the task to mark as completed: ");
    io::stdout().flush().unwrap();

    let mut task_id_str = String::new();
    io::stdin().read_line(&mut task_id_str).expect("Failed to read line");
    // FIX: Parsing target changed to i32 for consistency with Task.id
    let task_id: i32 = match task_id_str.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid task ID. Please enter a number.");
            return Ok(());
        }
    };

    let query = sqlx::query!(
        "UPDATE tasks SET completed = TRUE WHERE id = ?",
        task_id
    );

    let result = query.execute(pool).await?;

    if result.rows_affected() > 0 {
        println!("Task with ID {} marked as completed.", task_id);
    } else {
        println!("No task found with ID {}. Nothing updated.", task_id);
    }
    Ok(())
}

async fn delete_task(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    print!("Enter the ID of the task to delete: ");
    io::stdout().flush().unwrap();

    let mut task_id_str = String::new();
    io::stdin().read_line(&mut task_id_str).expect("Failed to read line");
    // FIX: Parsing target changed to i32 for consistency with Task.id
    let task_id: i32 = match task_id_str.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid task ID. Please enter a number.");
            return Ok(());
        }
    };

    let query = sqlx::query!(
        "DELETE FROM tasks WHERE id = ?",
        task_id
    );

    let result = query.execute(pool).await?;

    if result.rows_affected() > 0 {
        println!("Task with ID {} deleted successfully.", task_id);
    } else {
        println!("No task found with ID {}. Nothing deleted.", task_id);
    }
    Ok(())
}