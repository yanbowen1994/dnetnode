#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

extern crate foundationdb;
extern crate futures;
use self::futures::future::*;

pub struct Database {
    pub db:foundationdb::database::Database,
}
impl Database {
    pub fn new() -> Database {
// init fdb api
        let network = foundationdb::init().expect("failed to initialize Fdb client");

        let handle = std::thread::spawn(move || {
            let error = network.run();

            if let Err(error) = error {
                panic!("fdb_run_network: {}", error);
            }
        });

// wait for the network thread to be started
        network.wait();

        let db:foundationdb::database::Database =
            foundationdb::cluster::Cluster::new(foundationdb::default_config_path())
                .and_then(|cluster| cluster.create_database())
                .wait().expect("failed to create Cluster");
        Database {
            db,
        }
    }

    pub fn set(&self, key:&Vec<&str>, value:&[u8]) {
        let string_key = vec_to_key(key);
        let bytes_key = string_key.as_bytes();

        let trx = self.db.create_trx().expect("failed to create transaction");
        trx.set(bytes_key, value); // errors will be returned in the future result
        trx.commit()
            .wait()
            .expect("Fdb failed to set");
    }

    pub fn set_vec_str(&self, key:&Vec<&str>, value:&Vec<String>) {
        let mut new_value = Vec::new();
        for word in value.iter() {
            new_value.push(&word[..]);
        }
        let value_str = vec_to_key(&new_value);
        self.set(key, value_str.as_bytes())
    }

    pub fn get(&self, key:&Vec<&str>) -> String {
        let string_key = vec_to_key(key);
        let bytes_key = string_key.as_bytes();

        let trx = self.db.create_trx().expect("failed to create transaction");
        let result = trx.get(bytes_key, true).wait().expect("failed to read world from hello");

        let value: &[u8] = result.value()
            .expect("Fdb failed to get") // unwrap the error
            .unwrap();   // unwrap the option
        String::from_utf8_lossy(value).to_string()
    }

    pub fn clear(&self, key:&Vec<&str>) {
        let string_key = vec_to_key(key);
        let bytes_key = string_key.as_bytes();
        let trx = self.db.create_trx().expect("failed to create transaction");
        trx.clear(bytes_key);
        trx.commit()
            .wait()
            .expect("Fdb failed to clear");
    }

    pub fn clear_range(&self, key:&Vec<&str>) {
        let string_key = vec_to_key(key);
        let start = string_key.as_bytes();
        let end = (string_key.to_owned()[0..string_key.len() - 1].to_string() + "\x01");
        let trx = self.db.create_trx().expect("failed to create transaction");
        trx.clear_range(start, end.as_bytes());
        trx.commit()
            .wait()
            .expect("Fdb failed to clear");
    }
}

pub fn vec_to_key(input:&Vec<&str>) -> String {
    let mut output = String::new();
    for i in input {
        output += &("\x02".to_string() + *i + "\x00");
    };
    return output;
}

#[test]
fn test_set_get() {
    let api = FdbApi::new();

    let key = vec!["name", "name"];

    api.set(&key, b"qwe");

    let a = api.get(&key);
    assert_eq!(&a, "qwe");
}