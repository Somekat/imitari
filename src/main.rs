#![deny(clippy::all)]
use std::{collections::HashMap, time::Duration};

use actix_files::Files;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_multipart::Multipart;
use actix_ratelimit::{MemoryStore, MemoryStoreActor, RateLimiter};
use actix_web::{
	delete,
	dev::{Body, ServiceResponse},
	error,
	get,
	http::StatusCode,
	middleware,
	middleware::{
		errhandlers::{ErrorHandlerResponse, ErrorHandlers},
		Logger,
	},
	post,
	web,
	App,
	Error,
	HttpRequest,
	HttpResponse,
	HttpServer,
	Result,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use async_std::prelude::*;
use futures::{StreamExt, TryStreamExt};
use web::Payload;
mod auth;
use rand::Rng;
use tera::Tera;
mod id;

mod models;
use actix_cors::Cors;
use lazy_static::lazy_static;
use serde_json::json;

lazy_static! {
	pub static ref BASE_URL: String = std::env::var("BASE_URL").expect("BASE_URL not set");
	pub static ref AUTH_TOKEN: String = std::env::var("AUTH_TOKEN").expect("No AUTH_TOKEN set");
	pub static ref AUTH_USER: String = std::env::var("AUTH_USER").expect("No AUTH_USER set");
	pub static ref AUTH_PASSWORD: String =
		std::env::var("AUTH_PASSWORD").expect("No AUTH_PASSWORD set");
	pub static ref NAME: String = std::env::var("NAME").expect("no NAME set");
}

#[delete("/{token}", wrap = "HttpAuthentication::bearer(auth::validator)")]
async fn delete_file(file: web::Path<String>) -> Result<HttpResponse, Error> {
	let filename = file.into_inner();
	async_std::fs::remove_file(format!("./static/images/{}", filename)).await?;
	Ok(HttpResponse::Ok().json(json!({
		"message": "deleted file"
	})))
}

#[delete(
	"/delete/{token}",
	wrap = "HttpAuthentication::bearer(auth::validator)"
)]
async fn delete_get(file: web::Path<String>) -> Result<HttpResponse, Error> {
	let filename = file.into_inner();
	async_std::fs::remove_file(format!("./static/images/{}", filename)).await?;
	Ok(HttpResponse::Ok().json(json!({
		"message": "deleted file"
	})))
}

#[get("/login")]
async fn login(
	tmpl: web::Data<tera::Tera>,
	id: Identity,
	query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
	let mut login = false;
	let user = match id.identity() {
		Some(_u) => true,
		None => false,
	};
	if !user {
		println!("{:?}", query);
		if let Some(us) = query.get("username") {
			if let Some(p) = query.get("password") {
				if *us.to_string() == *AUTH_USER && *p.to_string() == *AUTH_PASSWORD {
					id.remember(us.to_string());
					login = true; // <- remember identity
				};
			};
		};
	} else {
		login = true;
	}
	println!("what");
	println!("{}", login);
	if login {
		Ok(HttpResponse::Found().header("location", "/ui").finish())
	} else {
		let temp = tmpl.render("login.html", &tera::Context::new()).unwrap();
		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(temp))
	}
}

#[get("/logout")]
async fn logout(id: Identity) -> HttpResponse {
	id.forget(); // <- remove identity
	HttpResponse::Found().header("location", "/").finish()
}

#[post("/image", wrap = "HttpAuthentication::bearer(auth::validator)")]
async fn file_save_rest(req: HttpRequest, mut payload: Payload) -> Result<HttpResponse, Error> {
	// let filename = parse_filename_from_uri(&req.uri().to_string())
	//     .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
	//let header = req
	//.headers()
	//.get("Content-Type")
	//.ok_or_else(|| actix_web::error::ParseError::Incomplete)?;

	let header = req
		.headers()
		.get("Content-Type")
		.ok_or(actix_web::error::ParseError::Incomplete)?;
	let file_fmt = header.to_str().unwrap().replace("image/", "");
	let filename = format!("{}.{}", id::PostId::generate().to_string(), file_fmt);
	let re =
		regex::Regex::new(r"([a-zA-Z0-9\s_\\.\-\(\):])+(.webp|.jpeg|.png|.gif|.jpg|.tiff|.bmp)$")
			.unwrap();
	let valid;
	if re.is_match(&filename) {
		let filepath = format!("./static/images/{}", sanitize_filename::sanitize(&filename));
		let mut f = async_std::fs::File::create(filepath).await?;
		while let Some(chunk) = payload.next().await {
			let data = chunk.unwrap();
			f.write_all(&data).await?;
		}
		valid = true;
	} else {
		valid = false;
	};
	if valid {
		Ok(HttpResponse::Ok().json(json!({
			"url": format!("{}/images/{}", *BASE_URL, filename),
			"deletion_url": format!("{}/delete/{}", *BASE_URL,filename)
		})))
	} else {
		Ok(HttpResponse::BadRequest().json(json!({
			"message": "No valid Image"
		})))
	}
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
	//let mut f_n = "".to_string();
	let mut f_n;
	let mut valid = false;
	let mut filevec = Vec::new();
	while let Ok(Some(mut field)) = payload.try_next().await {
		//let content_type = field
		//.content_disposition()
		//.ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
		let content_type = field
			.content_disposition()
			.ok_or(actix_web::error::ParseError::Incomplete)?;
		println!("{:?}", content_type);
		//let filename = content_type
		//.get_filename()
		//.ok_or_else(|| actix_web::error::ParseError::Incomplete)?;
		let filename = content_type
			.get_filename()
			.ok_or(actix_web::error::ParseError::Incomplete)?;
		let re = regex::Regex::new(
			r"([a-zA-Z0-9\s_\\.\-\(\):])+(.webp|.jpeg|.png|.gif|.jpg|.tiff|.bmp)$",
		)
		.unwrap();
		if re.is_match(filename) {
			let out = filename.split('.').collect::<Vec<&str>>()[1];
			let filename = format!("{}.{}", id::PostId::generate().to_string(), out);
			println!("{}", filename);
			let filepath = format!("./static/images/{}", sanitize_filename::sanitize(&filename));
			f_n = filename.to_string();
			let mut f = async_std::fs::File::create(filepath).await?;
			while let Some(chunk) = field.next().await {
				let data = chunk.unwrap();
				f.write_all(&data).await?;
			}
			filevec.push(json!({
				"url": format!("{}/images/{}", *BASE_URL, f_n) ,
				"deletion_url": format!("{}/delete/{}", *BASE_URL,f_n)
			}));
			valid = true;
		} else {
			valid = false;
		}
	}
	//let uri = req.uri();
	if valid {
		Ok(HttpResponse::Ok().json(json!({ "images": filevec })))
	} else {
		Ok(HttpResponse::BadRequest().json(json!({
			"message": "No valid Image"
		})))
	}
}

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
	let f = async_std::fs::read("./static/images/home.png").await?;
	Ok(HttpResponse::Ok().content_type("image/png").body(f))
}

#[get("/ui")]
async fn upload_ui(tmpl: web::Data<tera::Tera>, id: Identity) -> Result<HttpResponse, Error> {
	println!("{:?}", id.identity());
	let user = match id.identity() {
		Some(_u) => true,
		None => false,
	};
	println!("{}", user);
	if user {
		let temp = tmpl
			.render("upload.html", &tera::Context::new())
			.map_err(|_| error::ErrorInternalServerError("Template error"))?;

		Ok(HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.body(temp))
	} else {
		Ok(HttpResponse::Found().header("location", "/login").finish())
	}
}

fn error_handlers() -> ErrorHandlers<Body> {
	ErrorHandlers::new()
		.handler(StatusCode::METHOD_NOT_ALLOWED, method_not_allowed)
		.handler(StatusCode::NOT_FOUND, not_found)
}

fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
	Ok(ErrorHandlerResponse::Response(
		res.into_response(
			HttpResponse::NotFound()
				.json(json!({"message": "not founf"}))
				.into_body(),
		),
	))
}

fn method_not_allowed<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
	let resa = res.request().clone();
	Ok(ErrorHandlerResponse::Response(
		res.into_response(
			HttpResponse::NotFound()
				.json(json!({
					"message":
						format!(
							"{} is not allowed for url {}",
							resa.method().to_string(),
							resa.uri().to_string()
						)
				}))
				.into_body(),
		),
	))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
	std::env::set_var("RUST_BACKTRACE", "1");
	println!("Loading Env Vars");
	dotenv::dotenv().ok();
	println!("Starting Server......");

	let start = std::env::var("URL").expect("*sigh* Where is URL? Please set it in your .env file");
	let port = std::env::var("PORT").expect("*sigh* Where is PORT? Please set it in your .env file");
	env_logger::init();
	HttpServer::new(|| {
		let tera = Tera::new("templates/**/*").unwrap();
		let store = MemoryStore::new();
		let auth_conf = models::Auth {
			user: AUTH_USER.as_str().to_string(),
			password: AUTH_PASSWORD.as_str().to_string(),
		};
		let protect_form = Cors::default().allow_any_origin();
		let private_key = rand::thread_rng().gen::<[u8; 32]>();
		println!("Data Ready");
		App::new()
			.data(tera)
			.wrap(middleware::Compress::default())
			.service(Files::new("/images", "static/images"))
			.service(Files::new("/static", "public"))
			.wrap(
				RateLimiter::new(MemoryStoreActor::from(store).start())
					.with_interval(Duration::from_secs(60))
					.with_max_requests(30),
			)
			.data(auth_conf)
			.service(index)
			.service(
				web::resource("/ui/upload")
					.route(web::post().to(save_file))
					.wrap(protect_form),
			)
			.service(upload_ui)
			.service(file_save_rest)
			.service(delete_file)
			.service(delete_get)
			.wrap(IdentityService::new(
				CookieIdentityPolicy::new(&private_key)
					.name("cdn")
					.secure(false),
			))
			.service(
				web::resource("/upload")
					.route(web::post().to(save_file))
					.wrap(HttpAuthentication::bearer(auth::validator)),
			)
			.service(login)
			.service(logout)
			.service(web::scope("").wrap(error_handlers()))
			.wrap(Logger::default())
	})
	.workers(1)
	.bind(format!("{}:{}", start, port))
	.unwrap()
	.run()
	.await
}
