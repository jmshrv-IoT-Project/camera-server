use super::schema::{cameras, users_cameras};
use super::CameraServerDbConn;
use crate::{api_error::ApiError, camera::Camera, user_tokens};
use diesel::prelude::*;
use diesel::{self};
use rocket::get;
use rocket::http::Status;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Queryable, AsChangeset, Deserialize, Serialize)]
#[table_name = "users_cameras"]
pub struct UsersCamera {
    pub users_cameras_id: i32,
    pub camera_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name = "users_cameras"]
pub struct InsertableUsersCamera {
    pub camera_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<UsersCamera>> {
    users_cameras::table.load::<UsersCamera>(&*connection)
}

pub fn get(users_cameras_id: i32, connection: &PgConnection) -> QueryResult<UsersCamera> {
    users_cameras::table
        .find(users_cameras_id)
        .get_result::<UsersCamera>(connection)
}

pub fn insert(
    users_camera: InsertableUsersCamera,
    connection: &PgConnection,
) -> QueryResult<UsersCamera> {
    diesel::insert_into(users_cameras::table)
        .values(users_camera)
        .get_result(connection)
}

pub fn update(
    users_cameras_id: i32,
    users_camera: UsersCamera,
    connection: &PgConnection,
) -> QueryResult<UsersCamera> {
    diesel::update(users_cameras::table.find(users_cameras_id))
        .set(&users_camera)
        .get_result(connection)
}

pub fn delete(users_cameras_id: i32, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(users_cameras::table.find(users_cameras_id)).execute(connection)
}

pub fn get_users_cameras(
    user_id: uuid::Uuid,
    connection: &PgConnection,
) -> QueryResult<Vec<Camera>> {
    users_cameras::table
        .filter(users_cameras::user_id.eq(user_id))
        .inner_join(cameras::table.on(cameras::camera_id.eq(users_cameras::camera_id)))
        .select((cameras::camera_id, cameras::name))
        .load(connection)
}

/// Checks if the user in user_token has access to the camera with an ID of camera_id_string.
/// Returns an empty Ok() if access is allowed, returns ApiError if the user isn't allowed or if something else goes wrong.
pub fn check_if_user_has_access_to_camera(
    conn: &CameraServerDbConn,
    user_token: &user_tokens::UserToken,
    camera_id_string: &String,
) -> Result<(), ApiError> {
    let camera_id = uuid::Uuid::parse_str(camera_id_string).map_err(|error| {
        println!(
            "Failed to parse camera id into UUID: Input was {}, error was {}",
            camera_id_string, error
        );
        ApiError {
            error: "Failed to parse camera ID string",
            status: Status::UnprocessableEntity,
        }
    })?;

    let users_cameras_list = get_users_cameras(user_token.user_id, conn).map_err(|error| {
        println!(
            "Failed to get list of user's cameras! The error was {}",
            error
        );
        ApiError {
            error: "Failed to get list of owned cameras",
            status: Status::InternalServerError,
        }
    })?;

    // If the user doesn't have access to the camera (camera id is not returned by users_cameras), return an error
    if !users_cameras_list
        .iter()
        .any(|users_camera| users_camera.camera_id == camera_id)
    {
        return Err(ApiError {
            error: "User does not have access to camera",
            status: Status::Unauthorized,
        });
    }

    Ok(())
}

/// Returns a list of camera IDs for a user's cameras
#[get("/ListCameras")]
pub fn list_cameras(
    conn: CameraServerDbConn,
    user_token: user_tokens::UserToken,
) -> Result<Json<Vec<Camera>>, ApiError> {
    let camera_list = get_users_cameras(user_token.user_id, &conn).map_err(|error| {
        println!(
            "Failed to get user's cameras for user ID {}. The error was {}",
            user_token.user_id, error
        );
        ApiError {
            error: "Database failed to get list of cameras",
            status: Status::InternalServerError,
        }
    })?;

    Ok(Json(camera_list))
}
