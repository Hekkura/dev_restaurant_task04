use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;
use chrono::prelude::*;

#[derive(Debug)]
struct Food {
    id: i64,
    name: String,
    stock: i32,
    price: i32,
}
#[derive(Debug)]
struct Foods {
    inner: HashMap<i64, Food>,
}

impl Foods {
    fn new() -> Self{ 
        Self{
            inner: HashMap::new(),
        }
    }

    fn edit(&mut self, id:i64, name:&str, stock:i32, price:i32){
        self.inner.insert(
            id,
            Food { id, name: name.to_string(), stock, price },
        );
    }

    fn next_id(&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop() {
            Some(id) => return id + 1,
            None => return 1,
        }
    }

    fn add(&mut self, food: Food) {
        self.inner.insert(food.id, food);
    }

    fn into_vec(mut self) -> Vec<Food> {
        let mut foods: Vec<_> = self.inner.drain().map(|kv| kv.1).collect();
        foods.sort_by_key(|fd| fd.id);
        return foods
    }

    fn search(&self, name: &str) -> Vec<&Food> {
        self.inner 
            .values()
            .filter(|food| food.name.to_lowercase().contains(&name.to_lowercase()))
            .collect()
    }

    fn remove(&mut self, id: i64) -> Option<Food> {
        self.inner.remove(&id)
    }
}


// #[derive(Debug)]
// struct Report{
//     id:i64,
//     date: DateTime<Local>,
//     sell: i32,
//     income: i32,
// }
// #[derive(Debug)]
// struct Reports {
//     inner: HashMap<i64, Report>
// }

#[derive(Error, Debug)]
enum ParseError {
    #[error("id must be a number: {0}")]
    InvalidId(#[from] std::num::ParseIntError),
    
    #[error("empty record")]
    EmptyRecord,
    
    #[error("missing field: {0}")]
    MissingField(String),
}

fn parse_food(food: &str) -> Result<Food, ParseError> {
    let fields: Vec<&str> = food.split(',').collect();

    let id = match fields.get(0){
        Some(id) => i64::from_str_radix(id,10)?,
        None => return Err(ParseError::EmptyRecord),
    };

    let name = match fields.get(1).filter(|name| **name !="") {
        Some(name) => name.to_string(),
        None => return Err(ParseError::MissingField("name".to_owned())),
    };

    let stock = match fields.get(2){
        Some(stock) => i32::from_str_radix(stock,10)?,
        None =>return Err(ParseError::EmptyRecord),
    };

    let price = match fields.get(3) {
        Some(price) => i32::from_str_radix(price,10)?,
        None =>return Err(ParseError::EmptyRecord),
    };

    return Ok(Food {id, name, stock, price})
}

fn parse_foods(foods: String, verbose: bool) -> Foods {
    let mut fds = Foods::new();

    for (num, food) in foods.split('\n').enumerate() {
        if food != "" {
            match parse_food(food) {
                Ok(fd) => fds.add(fd),
                Err(e) => {
                    if verbose {
                        println!("
                        Error on line number {}:{}\n > \"{}\"\n",
                        num+1,
                        e,
                        food
                        );
                    }
                }
            }
        }
    }
    return fds
}

fn load_foods(file_name: PathBuf, verbose: bool) -> std::io::Result<Foods> {
    let mut file = File::open(file_name)?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    return Ok(parse_foods(buffer, verbose))
}

fn save_foods(file_name: PathBuf, foods:Foods) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_name)?;

    file.write(b"id,name,stock,price\n")?;

    for food in foods.into_vec().into_iter() {
        let line = format!("{},{},{},{}\n", food.id, food.name, food.stock, food.price);
        file.write(line.as_bytes())?;
    }
    file.flush()?;
    return Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(about= "Contact Manager Application")]
struct Opt {
    #[structopt(short, parse(from_os_str), default_value = "food.csv")]
    data_file: PathBuf,
    #[structopt(subcommand)]
    cmd : Command,
    #[structopt(short, help = "verbose")]
    verbose: bool,
}

#[derive(StructOpt, Debug)]
enum Command {
    Add{
        name: String,
        stock: i32,
        price: i32,
    },
    Edit {
        id: i64,
        name: String,
        stock: i32,
        price: i32,
    },
    List{},
    Remove {
        id: i64,
    },
    Search {
        query : String,
    },

}

fn run(opt: Opt) -> Result <(), std::io::Error> {
    match opt.cmd {

        Command::Add{ name, stock, price} => {
            let mut fds = load_foods(opt.data_file.clone(), opt.verbose)?;
            let next_id = fds.next_id();
            fds.add(Food{
                id: next_id,
                name,
                stock,
                price,
            });
            save_foods(opt.data_file, fds)?;
        }
        
        Command::Edit {id, name, stock, price} => {
            let mut fds = load_foods(opt.data_file.clone(), opt.verbose)?;
            fds.edit(id, &name, stock, price);
            save_foods(opt.data_file, fds)?; 
        }


        Command::List { .. } => {
            let fds = load_foods(opt.data_file.clone(), opt.verbose)?;
            for food in fds.into_vec() {
                println!("{:?}", food);
            }
        }
        Command::Remove {id} => {
            let mut fds = load_foods(opt.data_file.clone(), opt.verbose)?;
            fds.remove(id);
            save_foods(opt.data_file, fds)?;
        }
        Command::Search { query } => {
            let fds = load_foods(opt.data_file.clone(), opt.verbose)?;
            let results = fds.search(&query);
            if results.is_empty() {
                println!("No records found");
            } else {
                for fd in results {
                    println!("{:?}", fd);
                }
            }
        }
    }
    return Ok(())
}


fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run (opt) {
        println!("An error occured: {}", e);
    }
}
