use im::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub nickname: String,
}

pub trait UsersRepository {
    fn get(&self, id: u64) -> Option<User>;
    fn get_many<I>(&self, ids: I) -> Vec<User>
    where
        I: IntoIterator<Item = u64>;
    fn search_by_nickname(&self, query: &str) -> Vec<u64>;
}

#[derive(Clone, Debug, Default)]
pub struct ImUsersRepository {
    users: HashMap<u64, User>,
}

impl ImUsersRepository {
    pub fn new<U>(users: U) -> Self
    where
        U: IntoIterator<Item = User>,
    {
        let users_map = users.into_iter().map(|user| (user.id, user)).collect();
        Self { users: users_map }
    }
}

impl UsersRepository for ImUsersRepository {
    fn get(&self, id: u64) -> Option<User> {
        self.users.get(&id).cloned()
    }

    fn get_many<I>(&self, ids: I) -> Vec<User>
    where
        I: IntoIterator<Item = u64>,
    {
        ids.into_iter()
            .filter_map(|id| self.users.get(&id).cloned())
            .collect()
    }

    fn search_by_nickname(&self, query: &str) -> Vec<u64> {
        let query = query.to_lowercase();
        self.users
            .values()
            .filter(|user| user.nickname.to_lowercase().contains(&query))
            .map(|user| user.id)
            .collect()
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_repository() -> ImUsersRepository {
        ImUsersRepository::new([
            User {
                id: 1,
                nickname: "Alice".into(),
            },
            User {
                id: 2,
                nickname: "Bob".into(),
            },
            User {
                id: 3,
                nickname: "Alicia".into(),
            },
        ])
    }

    #[test]
    fn get_returns_exact_user() {
        let repo = sample_repository();

        let user = repo.get(2);

        assert_eq!(
            user,
            Some(User {
                id: 2,
                nickname: "Bob".into(),
            })
        );
    }

    #[test]
    fn get_many_preserves_input_order_and_skips_missing() {
        let repo = sample_repository();

        let users = repo.get_many([3, 42, 1]);

        assert_eq!(
            users,
            vec![
                User {
                    id: 3,
                    nickname: "Alicia".into(),
                },
                User {
                    id: 1,
                    nickname: "Alice".into(),
                },
            ]
        );
    }

    #[test]
    fn search_by_nickname_is_case_insensitive() {
        let repo = sample_repository();

        let ids = repo.search_by_nickname("ali");

        assert_eq!(ids, vec![1, 3]);
    }
}
