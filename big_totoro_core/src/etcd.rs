use crate::{
    config::Config,
    result::{AppError, AppResult},
};
use etcd_client::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;

const PREFIX: &str = "/server/";

pub type ServerName = String;
pub type ServerUri = String;

#[derive(Clone)]
pub struct Etcd {
    id: i64,
    servers: Arc<RwLock<HashMap<ServerName, ServerUri>>>,
}

impl Etcd {
    pub fn new(config: Arc<Config>) -> Self {
        let id = snowflake::SnowflakeIdBucket::new(1, 1).get_id();
        let etcd = Etcd {
            id,
            servers: Arc::new(RwLock::new(HashMap::new())),
        };

        let etcd1 = etcd.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = etcd1.run(config.clone()).await {
                    println!("etcd error => {:?}", e);
                    etcd1.servers.write().await.clear();
                }
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });

        etcd
    }

    fn get_name(&self) -> String {
        format!("{PREFIX}{}", self.id)
    }

    async fn get_servers(&self) -> Vec<ServerUri> {
        self.servers
            .read()
            .await
            .iter()
            .map(|i| i.1.to_string())
            .collect::<HashSet<String>>()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
    }

    async fn run(&self, config: Arc<Config>) -> AppResult<()> {
        let vec_uri = &config.etcd_uri;
        let mut client = Client::connect(vec_uri, None).await?;
        let resp = client.lease_grant(18, None).await?;
        let lease_id = resp.id();

        let mut lease_task_client = client.clone();
        let mut lease_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            while let Ok(_) = lease_task_client.lease_keep_alive(lease_id).await {
                interval.tick().await;
            }

            Ok::<(), AppError>(())
        });

        let server_task_self = self.clone();
        let mut server_task_client = client.clone();
        let mut server_task = tokio::spawn(async move {
            let (_, mut stream) = server_task_client
                .watch(
                    PREFIX,
                    Some(WatchOptions::new().with_all_keys().with_prefix()),
                )
                .await?;

            while let Some(resp) = stream.message().await? {
                for event in resp.events() {
                    if let Some(kv) = event.kv() {
                        let event_type = event.event_type();
                        let key = kv.key_str()?.to_string();
                        let value = kv.value_str()?.to_string();

                        match event_type {
                            EventType::Put => server_task_self
                                .servers
                                .write()
                                .await
                                .insert(key.clone(), value.clone()),
                            EventType::Delete => {
                                server_task_self.servers.write().await.remove(&key)
                            }
                        };
                    }
                }
                println!("{:?}", server_task_self.get_servers().await);
            }

            Ok::<(), AppError>(())
        });

        let key = self.get_name();
        let value = format!("http://{}:{}", config.local_ip, config.local_ws_server_port);
        client
            .put(key, value, Some(PutOptions::new().with_lease(lease_id)))
            .await?;

        let res = client
            .get(
                PREFIX,
                Some(GetOptions::new().with_all_keys().with_prefix()),
            )
            .await?;

        for kv in res.kvs() {
            let key = kv.key_str()?.to_string();
            let value = kv.value_str()?.to_string();
            self.servers
                .write()
                .await
                .insert(key.clone(), value.clone());
        }

        println!("{:?}", self.get_servers().await);
        tokio::select! {
            r = &mut lease_task =>{
                server_task.abort();

                match r {
                    Ok(r) => if let Err(r) = r {
                        return Err(r);
                    },
                    Err(e) => return Err(e.into()),
                }
            }
            r = &mut server_task =>{
                lease_task.abort();

                match r {
                    Ok(r) => if let Err(r) = r {
                        return Err(r);
                    },
                    Err(e) => return Err(e.into()),
                }
            }
        }

        Ok(())
    }
}
