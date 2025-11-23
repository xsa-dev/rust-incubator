use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, Result};

#[derive(Parser)]
#[command(author, version, about = "Simple SQLite-backed CLI for users and roles")]
struct Cli {
    /// Path to the SQLite database file
    #[arg(long, default_value = "roles.sqlite")] 
    database: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new role
    CreateRole {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "[]")]
        permissions: String,
    },
    /// Update role name or permissions
    UpdateRole {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        permissions: Option<String>,
    },
    /// Delete a role if no users rely on it
    DeleteRole {
        #[arg(long)]
        slug: String,
    },
    /// List all roles
    ListRoles,
    /// Show a single role
    GetRole {
        #[arg(long)]
        slug: String,
    },
    /// Create a new user and assign role
    CreateUser {
        #[arg(long)]
        name: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        role: String,
    },
    /// Update user name or email
    UpdateUser {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        email: Option<String>,
    },
    /// Delete a user
    DeleteUser {
        #[arg(long)]
        id: i64,
    },
    /// Assign role to user
    AssignRole {
        #[arg(long)]
        user_id: i64,
        #[arg(long)]
        role: String,
    },
    /// Remove role from user (requires user to keep at least one role)
    UnassignRole {
        #[arg(long)]
        user_id: i64,
        #[arg(long)]
        role: String,
    },
    /// List all users with their roles
    ListUsers,
    /// Show single user with roles
    GetUser {
        #[arg(long)]
        id: i64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut db = Db::new(&cli.database)?;
    db.ensure_schema()?;

    match cli.command {
        Command::CreateRole { slug, name, permissions } => db.create_role(&slug, &name, &permissions)?,
        Command::UpdateRole { slug, name, permissions } => db.update_role(&slug, name, permissions)?,
        Command::DeleteRole { slug } => db.delete_role(&slug)?,
        Command::ListRoles => db.list_roles()?,
        Command::GetRole { slug } => db.get_role(&slug)?,
        Command::CreateUser { name, email, role } => db.create_user(&name, &email, &role)?,
        Command::UpdateUser { id, name, email } => db.update_user(id, name, email)?,
        Command::DeleteUser { id } => db.delete_user(id)?,
        Command::AssignRole { user_id, role } => db.assign_role(user_id, &role)?,
        Command::UnassignRole { user_id, role } => db.unassign_role(user_id, &role)?,
        Command::ListUsers => db.list_users()?,
        Command::GetUser { id } => db.get_user(id)?,
    }

    Ok(())
}

struct Db {
    conn: Connection,
}

impl Db {
    fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        Ok(Self { conn })
    }

    fn ensure_schema(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS roles (
                slug TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                permissions TEXT NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users_roles (
                user_id INTEGER NOT NULL,
                role_slug TEXT NOT NULL,
                PRIMARY KEY(user_id, role_slug),
                FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY(role_slug) REFERENCES roles(slug) ON DELETE RESTRICT
            )",
            [],
        )?;
        Ok(())
    }

    fn create_role(&mut self, slug: &str, name: &str, permissions: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO roles (slug, name, permissions) VALUES (?1, ?2, ?3)",
            params![slug, name, permissions],
        )?;
        println!("Role '{slug}' created.");
        Ok(())
    }

    fn update_role(&mut self, slug: &str, name: Option<String>, permissions: Option<String>) -> Result<()> {
        let mut role = self.conn.query_row(
            "SELECT name, permissions FROM roles WHERE slug = ?1",
            params![slug],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        if let Some(new_name) = name {
            role.0 = new_name;
        }
        if let Some(new_perms) = permissions {
            role.1 = new_perms;
        }
        self.conn.execute(
            "UPDATE roles SET name = ?1, permissions = ?2 WHERE slug = ?3",
            params![role.0, role.1, slug],
        )?;
        println!("Role '{slug}' updated.");
        Ok(())
    }

    fn delete_role(&mut self, slug: &str) -> Result<()> {
        let users_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM users_roles WHERE role_slug = ?1",
            params![slug],
            |row| row.get(0),
        )?;
        if users_count > 0 {
            println!("Cannot delete role '{slug}' while it is assigned to users.");
            return Ok(());
        }
        let deleted = self.conn.execute("DELETE FROM roles WHERE slug = ?1", params![slug])?;
        if deleted == 0 {
            println!("Role '{slug}' not found.");
        } else {
            println!("Role '{slug}' deleted.");
        }
        Ok(())
    }

    fn list_roles(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT slug, name, permissions FROM roles ORDER BY slug")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;
        for row in rows {
            let (slug, name, perms) = row?;
            println!("{slug}: {name} | permissions={perms}");
        }
        Ok(())
    }

    fn get_role(&mut self, slug: &str) -> Result<()> {
        let role = self.conn.query_row(
            "SELECT slug, name, permissions FROM roles WHERE slug = ?1",
            params![slug],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        );
        match role {
            Ok((slug, name, perms)) => println!("{slug}: {name} | permissions={perms}"),
            Err(_) => println!("Role '{slug}' not found."),
        }
        Ok(())
    }

    fn create_user(&mut self, name: &str, email: &str, role: &str) -> Result<()> {
        self.ensure_role_exists(role)?;
        self.conn.execute(
            "INSERT INTO users (name, email) VALUES (?1, ?2)",
            params![name, email],
        )?;
        let user_id = self.conn.last_insert_rowid();
        self.assign_role(user_id, role)?;
        println!("User '{name}' created with id {user_id}.");
        Ok(())
    }

    fn update_user(&mut self, id: i64, name: Option<String>, email: Option<String>) -> Result<()> {
        let mut user = self.conn.query_row(
            "SELECT name, email FROM users WHERE id = ?1",
            params![id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        let Ok(mut existing) = user else {
            println!("User with id {id} not found.");
            return Ok(());
        };
        if let Some(new_name) = name {
            existing.0 = new_name;
        }
        if let Some(new_email) = email {
            existing.1 = new_email;
        }
        self.conn.execute(
            "UPDATE users SET name = ?1, email = ?2 WHERE id = ?3",
            params![existing.0, existing.1, id],
        )?;
        println!("User {id} updated.");
        Ok(())
    }

    fn delete_user(&mut self, id: i64) -> Result<()> {
        let deleted = self.conn.execute("DELETE FROM users WHERE id = ?1", params![id])?;
        if deleted == 0 {
            println!("User with id {id} not found.");
        } else {
            println!("User {id} deleted.");
        }
        Ok(())
    }

    fn assign_role(&mut self, user_id: i64, role: &str) -> Result<()> {
        self.ensure_role_exists(role)?;
        self.ensure_user_exists(user_id)?;
        self.conn.execute(
            "INSERT OR IGNORE INTO users_roles (user_id, role_slug) VALUES (?1, ?2)",
            params![user_id, role],
        )?;
        println!("Assigned role '{role}' to user {user_id}.");
        Ok(())
    }

    fn unassign_role(&mut self, user_id: i64, role: &str) -> Result<()> {
        self.ensure_user_exists(user_id)?;
        let role_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM users_roles WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        if role_count <= 1 {
            println!("User {user_id} must keep at least one role.");
            return Ok(());
        }
        let removed = self.conn.execute(
            "DELETE FROM users_roles WHERE user_id = ?1 AND role_slug = ?2",
            params![user_id, role],
        )?;
        if removed == 0 {
            println!("Role '{role}' not assigned to user {user_id}.");
        } else {
            println!("Removed role '{role}' from user {user_id}.");
        }
        Ok(())
    }

    fn list_users(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT id, name, email FROM users ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;
        for row in rows {
            let (id, name, email) = row?;
            let roles = self.roles_for_user(id)?;
            println!("{id}: {name} <{email}> | roles={roles}");
        }
        Ok(())
    }

    fn get_user(&mut self, id: i64) -> Result<()> {
        let user = self.conn.query_row(
            "SELECT name, email FROM users WHERE id = ?1",
            params![id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        match user {
            Ok((name, email)) => {
                let roles = self.roles_for_user(id)?;
                println!("{id}: {name} <{email}> | roles={roles}");
            }
            Err(_) => println!("User with id {id} not found."),
        }
        Ok(())
    }

    fn roles_for_user(&mut self, user_id: i64) -> Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT role_slug FROM users_roles WHERE user_id = ?1 ORDER BY role_slug",
        )?;
        let roles = stmt
            .query_map(params![user_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(roles.join(","))
    }

    fn ensure_role_exists(&mut self, slug: &str) -> Result<()> {
        let exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM roles WHERE slug = ?1",
            params![slug],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    fn ensure_user_exists(&mut self, id: i64) -> Result<()> {
        let exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM users WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }
}
