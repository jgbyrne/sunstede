use std::fs;
use jacl;
use jacl::EntryStruct;
use jacl::PropertyStruct;

#[derive(Debug)]
pub struct Server {
    key: String,
    addr: String,
    port: Option<i64>,
}

#[derive(Debug)]
pub struct Site {
    key: String,
    title: String,
    subtitle: String,
    servers: Vec<Server>,
}

#[derive(Debug)]
pub struct Config {
    sites: Vec<Site>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, ()>{
        let config = match fs::read_to_string(path) {
            Ok(config) => config,
            Err(_) => return Err(()),
        };

        let data = match jacl::read_string(&config) {
            Ok(data) => data,
            Err(e) => {
                eprint!("{}", e.render());
                return Err(());
            },
        };

        let mut sites: Vec<Site> = Vec::new();

        let root = data.root();
        match root.resolve_property("sites").as_ref()
                  .and_then(|sites| sites.as_entry_struct()) {

            Some(config_sites) => {
                for (key, entry) in config_sites.entries() {
                    let (key, site) = match (key, entry) {
                        (Some(key), Some(jacl::JaclStruct::Object(site))) => {
                            (key.clone(), site)
                        },
                        _ => {
                            eprintln!("Error: mis-specified site");
                            return Err(());
                        }
                    };

                    let title = match site.get_property("title") {
                        Some(jacl::Value::String(val)) => val.clone(),
                        _ => {
                            eprintln!("Error: {} has no site title", key);
                            return Err(());
                        }
                    };

                    let subtitle = match site.get_property("subtitle") {
                        Some(jacl::Value::String(val)) => val.clone(),
                        _ => {
                            eprintln!("Error: {} has no site subtitle", key);
                            return Err(());
                        }
                    };

                    let servers = match site.resolve_property("servers") {
                        Some(jacl::JaclStruct::Table(config_servers)) => {
                            let mut servers = Vec::new();
                            for (serv_key, server) in config_servers.entries() {
                                match (serv_key, server)  {
                                    (Some(serv_key), Some(server)) => {
                                        match server.as_property_struct() {
                                            Some(server) => {
                                                let addr = match server.get_property("addr") {
                                                    Some(jacl::Value::String(addr)) => addr.clone(),
                                                    _ => {
                                                        eprintln!("Error: {} server {}",
                                                                  key, serv_key);
                                                        return Err(());
                                                    },
                                                };

                                                let port = match server.get_property("port") {
                                                    Some(jacl::Value::Integer(port)) => { Some(*port) },
                                                    _ => {
                                                        None
                                                    },
                                                };

                                                servers.push(Server {
                                                    key: serv_key.clone(),
                                                    addr,
                                                    port,
                                                });
                                            },
                                            None => {
                                                eprintln!("Error: {} server {} - entry is a map",
                                                          key, serv_key);
                                                return Err(());
                                            },
                                        }
                                    },
                                    _ => {
                                        eprintln!("Error: mis-defined server for {}", key);
                                        return Err(());
                                    },
                                }
                            }
                            servers
                        },
                        _ => {
                            eprintln!("Error: {} has no server table", key);
                            return Err(());
                        }
                    };
                    sites.push( Site { key, title, subtitle, servers } );
                }
            },

            None => {
                eprintln!("Error: No sites structure in config");
                return Err(());
            }

        }

        Ok(Config { sites })
    }
}
