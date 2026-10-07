use serde::{Deserialize, Serialize};

/// POST /auth/google: the ID token Google's sign-in button hands the page
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignIn {
    pub credential: String,
}

/// reply to a sign-in: send `token` back as `Authorization: Bearer <token>`
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub token: String,
    pub me: Me,
}

/// GET /me
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Me {
    pub email: String,
    pub name: String,
    /// signed in with a psu.edu Google account
    pub psu_verified: bool,
    pub is_admin: bool,
    pub settings: Settings,
}

/// GET/PUT /me/settings
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Settings {
    pub mailing_list: bool,
    pub show_on_people: bool,
}

/// GET /admin/mailing-list
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Subscriber {
    pub email: String,
    pub name: String,
}

/// one person on the **People** page
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Person {
    pub id: i64,
    pub name: String,
    pub picture: Option<String>,
}

/// GET /people
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct People {
    pub count: i64,
    pub people: Vec<Person>,
}
