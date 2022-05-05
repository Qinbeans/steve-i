use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use crate::schema::{users, guilds, guild_users};

#[derive(Queryable,Insertable,AsChangeset)]
#[table_name="users"]
pub struct User {
    pub id: i64,
    pub name: Option<String>,
    pub tag: Option<String>,
    pub current_channel_id: Option<String>,
}

#[derive(Queryable,Insertable,AsChangeset)]
#[table_name="guilds"]
pub struct Guild {
      pub id: i64,
      pub name: Option<String>,
      pub prefix: Option<String>,
      pub owner_id: Option<i64>,
      pub cur_vc_id: Option<String>,
}

#[derive(Queryable,Insertable,AsChangeset)]
#[table_name="guild_users"]
pub struct GuildUser {
    pub user_id: i64,
    pub guild_id: i64,
}

pub fn get_users(conn: &MysqlConnection) -> Vec<User> {
    use crate::schema::users::dsl::*;
    users.load::<User>(conn).expect("Error loading users")
}

pub fn get_guilds(conn: &MysqlConnection) -> Vec<Guild> {
    use crate::schema::guilds::dsl::*;
    guilds.load::<Guild>(conn).expect("Error loading guilds")
}

pub fn get_guildusers(conn: &MysqlConnection) -> Vec<GuildUser> {
    use crate::schema::guild_users::dsl::*;
    guild_users.load::<GuildUser>(conn).expect("Error loading guild_users")
}

pub fn get_guild_by_id(conn: &MysqlConnection, tid: i64) -> Option<Guild> {
    use crate::schema::guilds::dsl::*;
    guilds.filter(id.eq(tid)).first::<Guild>(conn).ok()
}

impl User {
    pub fn new(conn: &MysqlConnection,id: i64, name: Option<String>, tag: Option<String>, current_channel_id: Option<String>) -> User {
        let n_user = User {
            id,
            name,
            tag,
            current_channel_id,
        };
        diesel::insert_into(users::table)
            .values(&n_user)
            .execute(conn)
            .expect("Error saving new user");
        n_user
    }
    pub fn insert(self, conn: &MysqlConnection){
        use crate::schema::users::dsl::*;
        diesel::insert_into(users).values(&self).execute(conn).expect("Error saving user");
    }
    pub fn change_channel(&mut self, channel_id: String){
        self.current_channel_id = Some(channel_id);
    }
    pub fn update(self, conn: &MysqlConnection){
        use crate::schema::users::dsl::*;
        diesel::update(users.find(self.id)).set(&self).execute(conn).expect("Error updating user");
    }
    pub fn remove(&self, conn: &MysqlConnection){
        use crate::schema::users::dsl::*;
        diesel::delete(users.find(self.id)).execute(conn).expect("Error deleting user");
    }
}

impl Guild {
    pub fn new(conn: &MysqlConnection, id: i64, name: Option<String>, prefix: Option<String>, owner_id: Option<i64>, cur_vc_id: Option<String>) -> Guild {
        let n_guild = Guild {
            id,
            name,
            prefix,
            owner_id,
            cur_vc_id,
        };
        diesel::insert_into(guilds::table)
            .values(&n_guild)
            .execute(conn)
            .expect("Error saving new guild");
        n_guild
    }
    pub fn insert(self, conn: &MysqlConnection){
        use crate::schema::guilds::dsl::*;
        diesel::insert_into(guilds).values(&self).execute(conn).expect("Error saving guild");
    }
    pub fn set_channel(mut self, channel_id: String){
        self.cur_vc_id = Some(channel_id);
    }
    pub fn set_name(mut self, name: String){
        self.name = Some(name);
    }
    pub fn set_prefix(mut self, conn: &MysqlConnection, prefix: String){
        self.prefix = Some(prefix);
        {
            use crate::schema::guilds::dsl::*;
            diesel::update(guilds.find(self.id)).set(&self).execute(conn).expect("Error updating guild");
        }
    }
    pub fn update(self, conn: &MysqlConnection){
        use crate::schema::guilds::dsl::*;
        diesel::update(guilds.find(self.id)).set(&self).execute(conn).expect("Error updating guild");
    }
    pub fn remove(&self, conn: &MysqlConnection){
        use crate::schema::guilds::dsl::*;
        diesel::delete(guilds.find(self.id)).execute(conn).expect("Error deleting guild");
    }
}

impl GuildUser {
    pub fn new(conn: &MysqlConnection, user_id: i64, guild_id: i64) -> GuildUser {
        let n_gu = GuildUser {
            user_id,
            guild_id,
        };
        diesel::insert_into(guild_users::table)
            .values(&n_gu)
            .execute(conn)
            .expect("Error saving new guild_user");
        n_gu
    }
    pub fn insert(self, conn: MysqlConnection){
        use crate::schema::guild_users::dsl::*;
        diesel::insert_into(guild_users).values(&self).execute(&conn).expect("Error saving guild_user");
    }
}