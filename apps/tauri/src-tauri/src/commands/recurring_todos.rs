use tauri::State;
use crate::database::pool::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RecurringTodo {
    pub id: String,
    pub todo_id: String,
    pub recurrence_rule: String,
    pub next_due_date: i64,
    pub created_at: i64,
}

fn advance_date(current: i64, rule: &str) -> i64 {
    match rule {
        "daily" => current + 86_400,
        "weekly" => current + 86_400 * 7,
        "biweekly" => current + 86_400 * 14,
        "monthly" => current + 86_400 * 30,
        "quarterly" => current + 86_400 * 91,
        "yearly" => current + 86_400 * 365,
        _ => current + 86_400 * 7, // default to weekly
    }
}

/// Set or update a recurrence rule on a todo.
#[tauri::command]
pub async fn set_recurrence(
    pool: State<'_, DbPool>,
    todo_id: String,
    recurrence_rule: String,
) -> Result<RecurringTodo, String> {
    let valid_rules = ["daily", "weekly", "biweekly", "monthly", "quarterly", "yearly"];
    if !valid_rules.contains(&recurrence_rule.as_str()) {
        return Err(format!("Invalid recurrence rule '{}'. Must be one of: {:?}", recurrence_rule, valid_rules));
    }

    let conn = pool.get().await.map_err(|e| e.to_string())?;

    // Check if the todo exists
    let has_due: Option<i64> = conn
        .query_row(
            "SELECT due_date FROM todos WHERE id = ?1",
            [&todo_id],
            |r| r.get(0),
        )
        .map_err(|e| format!("Todo not found: {}", e))?;

    let base_date = has_due.unwrap_or_else(|| chrono::Utc::now().timestamp());
    let next_due = advance_date(base_date, &recurrence_rule);
    let now = chrono::Utc::now().timestamp();
    let id = uuid::Uuid::new_v4().to_string();

    // Upsert: try insert, on conflict update
    let result = conn.execute(
        "INSERT INTO recurring_todos (id, todo_id, recurrence_rule, next_due_date, created_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(todo_id) DO UPDATE SET recurrence_rule=?3, next_due_date=?4",
        rusqlite::params![&id, &todo_id, &recurrence_rule, next_due, now],
    );
    drop(conn);

    match result {
        Ok(_) => Ok(RecurringTodo {
            id,
            todo_id,
            recurrence_rule,
            next_due_date: next_due,
            created_at: now,
        }),
        Err(e) => Err(format!("Failed to set recurrence: {}", e)),
    }
}

/// Remove recurrence from a todo.
#[tauri::command]
pub async fn remove_recurrence(pool: State<'_, DbPool>, todo_id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM recurring_todos WHERE todo_id = ?1", [&todo_id])
        .map_err(|e| e.to_string())?;
    drop(conn);
    Ok(())
}

/// List all recurring todo configurations.
#[tauri::command]
pub async fn list_recurring_todos(pool: State<'_, DbPool>) -> Result<Vec<RecurringTodo>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let result = {
        let mut stmt = conn
            .prepare("SELECT id, todo_id, recurrence_rule, next_due_date, created_at FROM recurring_todos")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(RecurringTodo {
                    id: r.get(0)?,
                    todo_id: r.get(1)?,
                    recurrence_rule: r.get(2)?,
                    next_due_date: r.get(3)?,
                    created_at: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            if let Ok(rt) = row {
                result.push(rt);
            }
        }
        result
    }; // stmt dropped here
    Ok(result)
}

/// Advance a recurring todo: update its due_date to the next occurrence and mark incomplete.
/// Returns error if the todo is archived (should not advance archived recurring todos).
pub fn advance_recurring_todo(conn: &rusqlite::Connection, todo_id: &str) -> Result<(), String> {
    // Check if the todo is archived — skip advancement for archived items
    let is_archived: bool = conn.query_row(
        "SELECT is_archived FROM todos WHERE id = ?1",
        [todo_id],
        |r| r.get::<_, i64>(0),
    ).map(|v| v != 0).unwrap_or(false);
    if is_archived {
        return Err("Todo is archived, not advancing recurrence".to_string());
    }

    let rt = conn.query_row(
        "SELECT recurrence_rule, next_due_date FROM recurring_todos WHERE todo_id = ?1",
        [todo_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
            ))
        },
    ).map_err(|e| format!("No recurrence found for todo {}: {}", todo_id, e))?;

    let (rule, _next_due) = rt;
    let now = chrono::Utc::now().timestamp();
    let new_due = advance_date(now, &rule);

    // Un-complete the todo and set the new due date
    conn.execute(
        "UPDATE todos SET is_completed = 0, due_date = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_due, now, todo_id],
    )
    .map_err(|e| format!("Failed to advance todo {}: {}", todo_id, e))?;

    // Update the recurring_todos next_due_date
    conn.execute(
        "UPDATE recurring_todos SET next_due_date = ?1 WHERE todo_id = ?2",
        rusqlite::params![new_due, todo_id],
    )
    .map_err(|e| format!("Failed to update recurring_todos for {}: {}", todo_id, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_recurring() -> RecurringTodo {
        RecurringTodo {
            id: "rt-1".into(),
            todo_id: "todo-1".into(),
            recurrence_rule: "weekly".into(),
            next_due_date: 1700000000 + 86_400 * 7,
            created_at: 1700000000,
        }
    }

    #[test]
    fn recurring_todo_serialize_roundtrip() {
        let rt = make_recurring();
        let json = serde_json::to_string(&rt).unwrap();
        let restored: RecurringTodo = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, restored);
    }

    #[test]
    fn recurring_todo_fields() {
        let rt = make_recurring();
        assert_eq!(rt.todo_id, "todo-1");
        assert_eq!(rt.recurrence_rule, "weekly");
    }

    #[test]
    fn advance_date_daily() {
        let result = advance_date(1700000000, "daily");
        assert_eq!(result, 1700000000 + 86_400);
    }

    #[test]
    fn advance_date_weekly() {
        let result = advance_date(1700000000, "weekly");
        assert_eq!(result, 1700000000 + 86_400 * 7);
    }

    #[test]
    fn advance_date_biweekly() {
        let result = advance_date(1700000000, "biweekly");
        assert_eq!(result, 1700000000 + 86_400 * 14);
    }

    #[test]
    fn advance_date_monthly() {
        let result = advance_date(1700000000, "monthly");
        assert_eq!(result, 1700000000 + 86_400 * 30);
    }

    #[test]
    fn advance_date_quarterly() {
        let result = advance_date(1700000000, "quarterly");
        assert_eq!(result, 1700000000 + 86_400 * 91);
    }

    #[test]
    fn advance_date_yearly() {
        let result = advance_date(1700000000, "yearly");
        assert_eq!(result, 1700000000 + 86_400 * 365);
    }

    #[test]
    fn advance_date_unknown_defaults_to_weekly() {
        let result = advance_date(1700000000, "unknown");
        assert_eq!(result, 1700000000 + 86_400 * 7);
    }

    #[test]
    fn advance_date_zero() {
        let result = advance_date(0, "daily");
        assert_eq!(result, 86_400);
    }

    #[test]
    fn clone_produces_equal() {
        let rt = make_recurring();
        assert_eq!(rt, rt.clone());
    }
}
