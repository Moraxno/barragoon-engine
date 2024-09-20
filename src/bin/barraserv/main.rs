#[macro_use] extern crate rocket;

use rocket::{get, launch, routes};
use rocket::fs::{FileServer, Options, relative};
use rocket_dyn_templates::{context, Template};

use barragoon_engine::common::navigation::{BOARD_HEIGHT, BOARD_WIDTH, RANK_NAMES};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/root")]
async fn root() -> Template {
    let coords = Iterator::zip(0..BOARD_HEIGHT, 0..BOARD_WIDTH);
    let color_coords = coords.map(|(rank, file)| {if (rank + file) % 2 == 0 { (rank, file, "light") } else { (rank, file, "dark") }}).collect::<Vec<(u8,u8,&str)>>();
    Template::render("root", context! { coords: color_coords})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8088)).merge(("template_dir", "src/bin/barraserv/templates")))
        // add templating system
        .attach(Template::fairing())
        // serve content from disk
        .mount("/public", FileServer::new(relative!("src/bin/barraserv/public"), Options::Missing | Options::NormalizeDirs))
        .mount("/", routes![index, root])       
}