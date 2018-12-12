extern crate std;

extern crate futures;
extern crate foundationdb;


use std::thread;

use futures::future::*;
use foundationdb::*;
use foundationdb::cluster::ClusterGet;
//use foundationdb::database::*;

pub struct FdbApi {
    pub db:Database,
}
impl FdbApi {
    pub fn new() -> FdbApi {
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

        let db:Database = Cluster::new(foundationdb::default_config_path())
            .and_then(|cluster| cluster.create_database())
            .wait().expect("failed to create Cluster");
        FdbApi {
            db,
        }
    }

    pub fn set(&self, key:&Vec<&str>, value:&'static [u8]) {
        let string_key = vec_to_key(key);
        let bytes_key = string_key.as_bytes();

        let trx = self.db.create_trx().expect("failed to create transaction");
        trx.set(bytes_key, value); // errors will be returned in the future result
        trx.commit()
            .wait()
            .expect("Fdb failed to set");
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