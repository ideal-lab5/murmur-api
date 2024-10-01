#[macro_use]
extern crate rocket;

// mod mongo_db;
// #[macro_use]
// extern crate bson;


use std::result;

// use mongodb::{bson::{doc, Document}, results::DeleteResult, error::Error};
use rocket_db_pools::mongodb::{bson::{self, doc, Bson, Document}, error::Error, results::DeleteResult, Collection};

use bcrypt::hash_with_salt;
use rocket::{futures::FutureExt, http::Status};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket_db_pools::{Database, mongodb::Client, Connection};
use serde::{Deserialize, Serialize};

const SALT: &str = "your-server-side-secret-salt";

// static mut DB: Option<MongoDbConnection> = None;

#[derive(Database)]
#[database("murmur")]
struct Db(Client); 

#[derive(Serialize, Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}
#[derive(Serialize, Deserialize)]
struct MMR {
	test: String,
	test2: String
}

#[get("/insert")]
async fn insert(db: Connection<Db>) {
	// db.database("admin").run_command(doc! {"ping": 1}, None).await;
	let test = String::from("abc");
	let test2 = String::from("cde");
	let doc = MMR{test, test2};
	let insert_result = db.database("Mmr").collection("mmrs").insert_one(doc, None).await;

	match insert_result {
		Err(e) => println!("Error inserting record : {e:?}"),
		Ok(insert) => {
			println!("succesfully inserted record, {insert:?}");
		}
	}
	// println!("Pinged your deployment. You successfully connected to MongoDB!");
	// Db.database("admin")
}

#[get("/delete")]
async fn delete(db: Connection<Db>) {

	let test = String::from("abc");
	let test2 = String::from("cde");
	let object = MMR{test, test2};

	let bson_try = bson::to_bson(&object);
	match bson_try {
		Err(e) => println!("Error turning object into bson {e:?}"),
		Ok(bson_object) => {

			let collection: Collection<MMR> = db.database("Mmr").collection("mmrs");

			let query = bson_object.as_document().unwrap();
		
			let delete_result = collection.delete_one(query.clone(), None).await;
			match delete_result {
					Err(e) => println!("Deletion error occurred: {e:?}"),
					Ok(success) => println!("Deletion Succeeded {success:?}")
				}
		}
	}
}

#[post("/login", data = "<login_request>")]
async fn login(login_request: Json<LoginRequest>, cookies: &CookieJar<'_>) -> &'static str {
	let username = &login_request.username;
	let password = &login_request.password;
	let seed = derive_seed(username, password);

	cookies.add(Cookie::new("username", username.clone()));
	cookies.add(Cookie::new("seed", seed.clone()));

	"User logged in, session started."
}

#[post("/create")]
async fn create(cookies: &CookieJar<'_>) -> Result<String, Status> {
	check_cookie(cookies, |_username, _seed| "create mmr called".to_string())
		.map_err(|_| Status::Forbidden)
}

#[post("/execute")]
async fn execute(cookies: &CookieJar<'_>) -> Result<String, Status> {
	check_cookie(cookies, |_username, _seed| "execute called".to_string())
		.map_err(|_| Status::Forbidden)
}

fn check_cookie(
	cookies: &CookieJar<'_>,
	callback: fn(username: &str, seed: &str) -> String,
) -> Result<String, ()> {
	let username = cookies.get("username");
	let seed = cookies.get("seed");
	match (username, seed) {
		(Some(username_cookie), Some(seed_cookie)) => {
			let username = username_cookie.value();
			let seed = seed_cookie.value();
			Ok(callback(username, seed))
		},
		_ => Err(()),
	}
}

fn derive_seed(password: &str, username: &str) -> String {
	hash_with_salt(format!("{}:{}", username, password), 4, SALT.as_bytes())
		.unwrap()
		.to_string()
}

#[launch]
async fn rocket() -> _ {
	// let connection = MongoDbConnection::new().await;
	// match connection {
	// 	Ok(c) => unsafe{db = Some(c)},
	// 	Err(e) => println!("DB Connection Failed: {e:?}")
	// }
	rocket::build().mount("/", routes![login, create, execute, insert, delete]).attach(Db::init())
}
