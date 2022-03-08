#[macro_use] extern crate rocket;

use rocket::serde::{Serialize, Deserialize};
use rocket::serde::json::Json;
use rusqlite::{Connection, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
struct Contact {
    id: usize,
    name: String,
    email: String,
    phone: String,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
struct ContactList {
    contacts: Vec<Contact>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct StatusMessage {
    message: String,
}

#[get("/")]
fn index() -> &'static str {
    let db_connection = Connection::open("contacts.sqlite").unwrap();

    db_connection.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
        id INTEGER PRIMARY KEY,
        name VARCHAR(64) NOT NULL,
        email VARCHAR(64) NOT NULL,
        phone VARCHAR(64) NOT NULL,
        message VARCHAR NOT NULL);",
        [],).unwrap();

    "rust_rest2, an api for ohioBSD.cloud"
}

#[get("/get_all_contacts")]
fn get_all_contacts() -> Result<Json<ContactList>, String> {
    let db_connection = match Connection::open("contacts.sqlite") {
        Ok(connection) => connection,
        Err(e) => return Err(format!("error: {:?}", e)),
    };
    let mut statement = match db_connection.prepare("SELECT id, name, phone, email, message FROM contacts;") {
        Ok(s) => s,
        Err(e) => return Err(format!("error: {:?}", e)),
    };

    let results = statement.query_map([], |row| {
        Ok(Contact {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            message: row.get(4)?,
        })
    });

    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();

            match collection {
                Ok(contacts) => Ok(Json(ContactList {contacts} )),
                Err(_) => Err(format!("Could not collect items")),
            }
        }
        Err(_) => Err(String::from("Failed to fetch Contacts")
        )
    }

}

#[post("/add_contact", format = "application/json", data="<contact>")]
fn add_contact(contact: Json<Contact>) -> Result<String, String> {

    let db_connection = match Connection::open("contacts.sqlite") {
        Ok(connection) => connection,
        Err(e) => {return Err(format!("Failed to connect to database, error code: {}", e))}
    };

    let mut statement = match db_connection.prepare(
        "INSERT INTO contacts (id, name, email, phone, message) values (null, $1, $2, $3, $4);") {
        Ok(s) => s,
        Err(_) => return Err("Failed to prepare query".into()),
    };
    let results = statement.execute([&contact.name,&contact.email,&contact.phone,&contact.message]);

    match results {
        Ok(results) => Ok(format!("ADDED CONTACT WITH name: {}, email: {}, phone: {}, message: {} \n\
                              {} rows changed",
                                  contact.name.clone(),
                                  contact.phone.clone(),
                                  contact.email.clone(),
                                  contact.message.clone(),
                                  results,
                                )),

        Err(e) => Err(format!("failed to add contact"))
    }

}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, add_contact, get_all_contacts])
}