use std::fs;
use jacl;
use jacl::EntryStruct;
use jacl::PropertyStruct;

#[derive(Debug)]
pub struct Server {
    pub key: String,
    pub addr: String,
    pub port: Option<i64>,
}

#[derive(Debug)]
pub struct View {
    pub key: String,
    pub mount: Option<String>,
}

#[derive(Debug)]
pub struct Site {
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub servers: Vec<Server>,
    pub views: Vec<View>,
}

#[derive(Debug)]
pub struct Config {
    pub sites: Vec<Site>,
}

pub enum ConfigError {
    Filesystem(String),
    Jacl(String),
    Logical(String),
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, ConfigError> {
        let config = match fs::read_to_string(path) {
            Ok(config) => config,
            Err(e) => {
                let msg = format!("Could not read config file: {}", e);
                return Err(ConfigError::Filesystem(msg));
            },
        };

        let data = match jacl::read_string(&config) {
            Ok(data) => data,
            Err(e) => {
                return Err(ConfigError::Jacl(format!("{}", e.render())));
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
                            return Err(ConfigError::Logical("Error: mis-specified site".to_string()));
                        }
                    };

                    let title = match site.get_property("title") {
                        Some(jacl::Value::String(val)) => val.clone(),
                        _ => {
                            let msg = format!("Error: {} has no site title", key);
                            return Err(ConfigError::Logical(msg));
                        }
                    };

                    let subtitle = match site.get_property("subtitle") {
                        Some(jacl::Value::String(val)) => val.clone(),
                        _ => {
                            let msg = format!("Error: {} has no site subtitle", key);
                            return Err(ConfigError::Logical(msg));
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
                                                        let msg = format!("Error: {} server {} has no `addr`", key, serv_key);
                                                        return Err(ConfigError::Logical(msg));
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
                                                let msg = format!("Error: {} server {} is a Map", key, serv_key);
                                                return Err(ConfigError::Logical(msg));
                                            },
                                        }

                                    },
                                    _ => {
                                        let msg = format!("Error: mis-defined server for {}", key);
                                        return Err(ConfigError::Logical(msg));
                                    },
                                }
                            }
                            servers
                        },
                        _ => { 
                            let msg = format!("Error: mis-defined server for {}", key);
                            return Err(ConfigError::Logical(msg));
                        }
                    };

                    let views = match site.resolve_property("views") {
                        Some(jacl::JaclStruct::Table(config_views)) => {
                            let mut views = Vec::new();

                            for (view_key, view) in config_views.entries() {
                                let (view_key, mount) = match (view_key, view)  {
                                    (Some(view_key), Some(view)) => {
                                        match view.as_property_struct() {
                                            Some(view) => {
                                                let mount = match view.get_property("mount") {
                                                    Some(jacl::Value::String(addr)) => {
                                                        Some(addr.clone())
                                                    },
                                                    _ => {
                                                        None
                                                    },
                                                };
                                                (view_key.clone(), mount)
                                            },
                                            None => {
                                                let msg = format!("Error: {} view {} - entry is a map", key, view_key);
                                                return Err(ConfigError::Logical(msg));
                                            },
                                        }
                                    },
                                    (Some(view_key), None) => {
                                        (view_key.clone(), None)
                                    }
                                    _ => {
                                        let msg = format!("Error: mis-defined view for {}", key);
                                        return Err(ConfigError::Logical(msg));
                                    },
                                };
                                views.push(View {key: view_key, mount});
                            }
                            views
                        },
                        _ => {
                            let msg = format!("Error: {} has no views table", key);
                            return Err(ConfigError::Logical(msg));
                        }
                    };

                    sites.push( Site { key, title, subtitle, servers, views } );
                }
            },

            None => {
                return Err(ConfigError::Logical("Error: No sites structure in config".to_string()));
            }

        }

        Ok(Config { sites })
    }
}
