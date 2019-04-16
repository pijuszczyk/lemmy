extern crate diesel;
use schema::{post, post_like};
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};
use {Crud, Likeable};

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name="post"]
pub struct Post {
  pub id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub creator_id: i32,
  pub community_id: i32,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="post"]
pub struct PostForm {
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub creator_id: i32,
  pub community_id: i32,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_like"]
pub struct PostLike {
  pub id: i32,
  pub post_id: i32,
  pub user_id: i32,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="post_like"]
pub struct PostLikeForm {
  pub post_id: i32,
  pub user_id: i32,
  pub score: i16
}

impl Crud<PostForm> for Post {
  fn read(conn: &PgConnection, post_id: i32) -> Result<Self, Error> {
    use schema::post::dsl::*;
    post.find(post_id)
      .first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, post_id: i32) -> Result<usize, Error> {
    use schema::post::dsl::*;
    diesel::delete(post.find(post_id))
      .execute(conn)
  }

  fn create(conn: &PgConnection, new_post: &PostForm) -> Result<Self, Error> {
    use schema::post::dsl::*;
      insert_into(post)
        .values(new_post)
        .get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, post_id: i32, new_post: &PostForm) -> Result<Self, Error> {
    use schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
  }
}

impl Likeable <PostLikeForm> for PostLike {
  fn read(conn: &PgConnection, post_id_from: i32) -> Result<Vec<Self>, Error> {
    use schema::post_like::dsl::*;
    post_like
      .filter(post_id.eq(post_id_from))
      .load::<Self>(conn) 
  }
  fn like(conn: &PgConnection, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    use schema::post_like::dsl::*;
    insert_into(post_like)
      .values(post_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(conn: &PgConnection, post_like_form: &PostLikeForm) -> Result<usize, Error> {
    use schema::post_like::dsl::*;
    diesel::delete(post_like
      .filter(post_id.eq(post_like_form.post_id))
      .filter(user_id.eq(post_like_form.user_id)))
      .execute(conn)
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  use Crud;
  use actions::community::*;
  use actions::user::*;
 #[test]
  fn test_crud() {
    let conn = establish_connection();

    let new_user = UserForm {
      name: "jim".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      admin: false,
      banned: false,
      updated: None
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: "test community_3".to_string(),
      title: "nada".to_owned(),
      description: None,
      category_id: 1,
      creator_id: inserted_user.id,
      removed: None,
      updated: None
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();
    
    let new_post = PostForm {
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      removed: None,
      locked: None,
      updated: None
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      published: inserted_post.published,
      removed: Some(false),
      locked: Some(false),
      updated: None
    };

    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1
    };

    let inserted_post_like = PostLike::like(&conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_post_like.published,
      score: 1
    };
    
    let read_post = Post::read(&conn, inserted_post.id).unwrap();
    let updated_post = Post::update(&conn, inserted_post.id, &new_post).unwrap();
    let like_removed = PostLike::remove(&conn, &post_like_form).unwrap();
    let num_deleted = Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, inserted_post);
    assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);

  }
}
