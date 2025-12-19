pub trait EventSourced<Ev: ?Sized> {
    fn apply(&mut self, event: &Ev);
}

pub mod user {
    use std::time::SystemTime;

    use super::{event, EventSourced};

    #[derive(Debug)]
    pub struct User {
        pub id: Id,
        pub name: Option<Name>,
        pub online_since: Option<SystemTime>,
        pub created_at: CreationDateTime,
        pub last_activity_at: LastActivityDateTime,
        pub deleted_at: Option<DeletionDateTime>,
    }

    impl EventSourced<event::UserCreated> for User {
        fn apply(&mut self, ev: &event::UserCreated) {
            let event::UserCreated { user_id, at } = ev;

            self.id = *user_id;
            self.created_at = *at;
            self.last_activity_at = LastActivityDateTime(at.0);
        }
    }

    impl EventSourced<event::UserNameUpdated> for User {
        fn apply(&mut self, ev: &event::UserNameUpdated) {
            let event::UserNameUpdated {
                user_id: _,
                name,
                at: _,
            } = ev;

            self.name = name.clone();
        }
    }

    impl EventSourced<event::UserBecameOnline> for User {
        fn apply(&mut self, ev: &event::UserBecameOnline) {
            let event::UserBecameOnline { user_id: _, at } = ev;

            self.online_since = Some(*at);
        }
    }

    impl EventSourced<event::UserBecameOffline> for User {
        fn apply(&mut self, ev: &event::UserBecameOffline) {
            let event::UserBecameOffline { user_id: _, at } = ev;

            self.online_since = None;
            self.last_activity_at = LastActivityDateTime(*at);
        }
    }

    impl EventSourced<event::UserDeleted> for User {
        fn apply(&mut self, ev: &event::UserDeleted) {
            let event::UserDeleted { user_id: _, at } = ev;

            self.deleted_at = Some(*at);
            self.last_activity_at = LastActivityDateTime(at.0);
        }
    }

    #[derive(Debug)]
    pub enum Event {
        Created(event::UserCreated),
        NameUpdated(event::UserNameUpdated),
        Online(event::UserBecameOnline),
        Offline(event::UserBecameOffline),
        Deleted(event::UserDeleted),
    }

    impl EventSourced<Event> for User {
        fn apply(&mut self, ev: &Event) {
            match ev {
                Event::Created(ev) => self.apply(ev),
                Event::NameUpdated(ev) => self.apply(ev),
                Event::Online(ev) => self.apply(ev),
                Event::Offline(ev) => self.apply(ev),
                Event::Deleted(ev) => self.apply(ev),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Id(pub u64);

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Name(pub Box<str>);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct CreationDateTime(pub SystemTime);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct LastActivityDateTime(pub SystemTime);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DeletionDateTime(pub SystemTime);
}

pub mod event {
    use std::time::SystemTime;

    use super::user;

    #[derive(Debug)]
    pub struct UserCreated {
        pub user_id: user::Id,
        pub at: user::CreationDateTime,
    }

    #[derive(Debug)]
    pub struct UserNameUpdated {
        pub user_id: user::Id,
        pub name: Option<user::Name>,
        pub at: SystemTime,
    }

    #[derive(Debug)]
    pub struct UserBecameOnline {
        pub user_id: user::Id,
        pub at: SystemTime,
    }

    #[derive(Debug)]
    pub struct UserBecameOffline {
        pub user_id: user::Id,
        pub at: SystemTime,
    }

    #[derive(Debug)]
    pub struct UserDeleted {
        pub user_id: user::Id,
        pub at: user::DeletionDateTime,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use crate::EventSourced;
    use super::{event, user::Event as UserEvent, user::*};

    fn empty_user() -> User {
        let epoch = SystemTime::UNIX_EPOCH;
        User {
            id: Id(0),
            name: None,
            online_since: None,
            created_at: CreationDateTime(epoch),
            last_activity_at: LastActivityDateTime(epoch),
            deleted_at: None,
        }
    }

    #[test]
    fn applies_individual_events() {
        let created_at = CreationDateTime(SystemTime::UNIX_EPOCH + Duration::from_secs(5));
        let mut user = empty_user();

        user.apply(&event::UserCreated {
            user_id: Id(1),
            at: created_at,
        });

        assert_eq!(user.id.0, 1);
        assert_eq!(user.created_at.0, created_at.0);
        assert_eq!(user.last_activity_at.0, created_at.0);

        let update_at = SystemTime::UNIX_EPOCH + Duration::from_secs(15);
        user.apply(&event::UserNameUpdated {
            user_id: Id(1),
            name: Some(Name("Ada".into())),
            at: update_at,
        });
        assert_eq!(user.name.as_ref().unwrap().0.as_ref(), "Ada");

        user.apply(&event::UserBecameOnline {
            user_id: Id(1),
            at: update_at,
        });
        assert_eq!(user.online_since, Some(update_at));

        let offline_at = SystemTime::UNIX_EPOCH + Duration::from_secs(45);
        user.apply(&event::UserBecameOffline {
            user_id: Id(1),
            at: offline_at,
        });
        assert_eq!(user.online_since, None);
        assert_eq!(user.last_activity_at.0, offline_at);

        let deletion_at = DeletionDateTime(SystemTime::UNIX_EPOCH + Duration::from_secs(90));
        user.apply(&event::UserDeleted {
            user_id: Id(1),
            at: deletion_at,
        });
        assert_eq!(user.deleted_at, Some(deletion_at));
        assert_eq!(user.last_activity_at.0, deletion_at.0);
    }

    #[test]
    fn applies_wrapped_events() {
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut user = empty_user();

        let events = [
            UserEvent::Created(event::UserCreated {
                user_id: Id(10),
                at: CreationDateTime(base_time),
            }),
            UserEvent::Online(event::UserBecameOnline {
                user_id: Id(10),
                at: base_time,
            }),
            UserEvent::Offline(event::UserBecameOffline {
                user_id: Id(10),
                at: base_time + Duration::from_secs(5),
            }),
        ];

        for ev in &events {
            user.apply(ev);
        }

        assert_eq!(user.id.0, 10);
        assert_eq!(user.created_at.0, base_time);
        assert_eq!(user.online_since, None);
        assert_eq!(
            user.last_activity_at.0,
            base_time + Duration::from_secs(5)
        );
    }
}
