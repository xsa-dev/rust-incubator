use std::marker::PhantomData;

mod post {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Id(u64);

    impl Id {
        pub fn new(value: u64) -> Self {
            Self(value)
        }

        pub fn get(&self) -> u64 {
            self.0
        }
    }

    impl From<u64> for Id {
        fn from(value: u64) -> Self {
            Self::new(value)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Title(String);

    impl Title {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }

        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl From<String> for Title {
        fn from(value: String) -> Self {
            Self::new(value)
        }
    }

    impl From<&str> for Title {
        fn from(value: &str) -> Self {
            Self::new(value)
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Body(String);

    impl Body {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }

        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl From<String> for Body {
        fn from(value: String) -> Self {
            Self::new(value)
        }
    }

    impl From<&str> for Body {
        fn from(value: &str) -> Self {
            Self::new(value)
        }
    }
}

mod user {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Id(u64);

    impl Id {
        pub fn new(value: u64) -> Self {
            Self(value)
        }

        pub fn get(&self) -> u64 {
            self.0
        }
    }

    impl From<u64> for Id {
        fn from(value: u64) -> Self {
            Self::new(value)
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct New;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Unmoderated;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Published;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Deleted;

#[derive(Debug)]
pub struct Post<State> {
    id: post::Id,
    author_id: user::Id,
    title: post::Title,
    body: post::Body,
    state: PhantomData<State>,
}

impl<State> Post<State> {
    pub fn id(&self) -> &post::Id {
        &self.id
    }

    pub fn author_id(&self) -> &user::Id {
        &self.author_id
    }

    pub fn title(&self) -> &post::Title {
        &self.title
    }

    pub fn body(&self) -> &post::Body {
        &self.body
    }
}

impl Post<New> {
    pub fn new(
        id: impl Into<post::Id>,
        author_id: impl Into<user::Id>,
        title: impl Into<post::Title>,
        body: impl Into<post::Body>,
    ) -> Self {
        Post {
            id: id.into(),
            author_id: author_id.into(),
            title: title.into(),
            body: body.into(),
            state: PhantomData,
        }
    }

    pub fn publish(self) -> Post<Unmoderated> {
        let Post {
            id,
            author_id,
            title,
            body,
            ..
        } = self;

        Post {
            id,
            author_id,
            title,
            body,
            state: PhantomData,
        }
    }
}

impl Post<Unmoderated> {
    pub fn allow(self) -> Post<Published> {
        let Post {
            id,
            author_id,
            title,
            body,
            ..
        } = self;

        Post {
            id,
            author_id,
            title,
            body,
            state: PhantomData,
        }
    }

    pub fn deny(self) -> Post<Deleted> {
        let Post {
            id,
            author_id,
            title,
            body,
            ..
        } = self;

        Post {
            id,
            author_id,
            title,
            body,
            state: PhantomData,
        }
    }
}

impl Post<Published> {
    pub fn delete(self) -> Post<Deleted> {
        let Post {
            id,
            author_id,
            title,
            body,
            ..
        } = self;

        Post {
            id,
            author_id,
            title,
            body,
            state: PhantomData,
        }
    }
}

fn main() {
    let post = Post::<New>::new(1_u64, 7_u64, "My first post", "Hello, world!");
    let post = post.publish();
    let post = post.allow();
    let _post = post.delete();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_flow_produces_published_post() {
        let post = Post::<New>::new(1_u64, 7_u64, "My first post", "Hello, world!");
        let post = post.publish().allow();

        assert_eq!(post.title().as_str(), "My first post");
        assert_eq!(post.body().as_str(), "Hello, world!");
        assert_eq!(post.id().get(), 1);
        assert_eq!(post.author_id().get(), 7);
    }

    #[test]
    fn deny_moves_post_to_deleted_state() {
        let post = Post::<New>::new(2_u64, 9_u64, "Pending post", "Needs review");
        let post = post.publish().deny();

        let _deleted: Post<Deleted> = post;
    }
}
