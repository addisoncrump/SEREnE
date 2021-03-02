use std::collections::HashMap;
use std::sync::Arc;

use bollard::container::{Config, ListContainersOptions, RemoveContainerOptions};
use bollard::errors::Error;
use bollard::models::{HostConfig, PortBinding, PortMap};
use bollard::Docker;
use log::*;
use thrussh_keys::key::PublicKey;
use thrussh_keys::PublicKeyBase64;
use tokio::net::TcpListener;

pub struct SandboxManager {
    docker: Arc<Docker>,
    map: HashMap<u64, Sandbox>,
}

impl SandboxManager {
    pub async fn new() -> Result<Self, Error> {
        Ok(SandboxManager {
            docker: Arc::new(Docker::connect_with_unix_defaults()?),
            map: HashMap::new(),
        })
    }

    async fn find_open(&self) -> Result<u16, anyhow::Error> {
        Ok(TcpListener::bind("0.0.0.0:0").await?.local_addr()?.port())
    }

    pub async fn create_sandbox(
        &mut self,
        id: u64,
        pubkey: PublicKey,
    ) -> Result<Option<u16>, anyhow::Error> {
        if self.map.contains_key(&id) {
            return Ok(None);
        }
        let sandbox = Sandbox::new(self.docker.clone(), pubkey, self.find_open().await?).await?;
        let port = sandbox.port;
        sandbox.start().await?;
        self.map.insert(id, sandbox);
        Ok(Some(port))
    }

    pub async fn destroy_sandbox(&mut self, id: u64) -> Result<bool, anyhow::Error> {
        let sandbox = self.map.remove(&id);
        if sandbox.is_some() {
            sandbox.unwrap().teardown().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn find_sandbox(&self, id: u64) -> Option<u16> {
        self.map.get(&id).map(|sandbox| sandbox.port)
    }

    pub async fn teardown(&self) -> Result<(), Error> {
        let mut filters = HashMap::new();
        filters.insert("ancestor", vec!["serene-sandbox"]);
        for container in self
            .docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await?
        {
            self.docker
                .remove_container(
                    &container.id.unwrap(),
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await?;
        }

        Ok(())
    }
}

pub struct Sandbox {
    docker: Arc<Docker>,
    id: String,
    port: u16,
}

impl Sandbox {
    async fn new(docker: Arc<Docker>, pubkey: PublicKey, port: u16) -> Result<Self, Error> {
        info!("initialising sandbox at port {}", port);
        let mut bindings = PortMap::new();
        bindings.insert(
            "22/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: None,
                host_port: Some(port.to_string()),
            }]),
        );
        let host_config = HostConfig {
            port_bindings: Some(bindings),
            ..Default::default()
        };

        let mut exposed = HashMap::new();
        exposed.insert("22/tcp".to_string(), HashMap::new());

        let config: Config<String> = Config {
            image: Some("serene-sandbox".to_string()),
            host_config: Some(host_config),
            env: Some(vec![format!(
                "SSH_KEY={} {}",
                pubkey.name(),
                pubkey.public_key_base64()
            )]),
            exposed_ports: Some(exposed),
            ..Default::default()
        };

        docker
            .create_container::<String, String>(None, config)
            .await
            .map(|container| Sandbox {
                docker,
                id: container.id,
                port,
            })
    }

    async fn start(&self) -> Result<(), Error> {
        info!("starting sandbox at port {}", self.port);
        self.docker
            .start_container::<String>(&self.id, None)
            .await?;
        Ok(())
    }

    async fn teardown(self) -> Result<(), Error> {
        info!("destroying sandbox at port {}", self.port);
        self.docker
            .remove_container(
                &self.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;
    use std::time::Duration;

    use thrussh_keys::key::KeyPair;
    use tokio::time::sleep;

    use super::*;

    use thrussh::{client, ChannelId};

    #[tokio::test]
    async fn launch_sandbox() -> Result<(), Box<dyn Error>> {
        env_logger::init();

        let keypair = Arc::new(
            KeyPair::generate_ed25519().expect("keypair generation is supposed to be stable!"),
        );
        let mut manager = SandboxManager::new().await?;

        let config = Arc::new(thrussh::client::Config::default());
        let client = TestClient {};

        let _sandbox = manager
            .create_sandbox(0, keypair.clone_public_key())
            .await?;

        sleep(Duration::from_secs(1)).await;

        let mut session = thrussh::client::connect(config, "localhost:31337", client)
            .await
            .unwrap();
        match session.authenticate_publickey("serene", keypair).await {
            Ok(true) => {}
            Ok(false) => {
                panic!("Auth failed; credentials rejected.")
            }
            Err(e) => {
                panic!("Auth failed: {:?}", e)
            }
        }
        session
            .disconnect(
                thrussh::Disconnect::ByApplication,
                "goodnight, sweet prince",
                "en_US.UTF8",
            )
            .await?;

        manager.teardown().await?;
        Ok(())
    }

    struct TestClient {}

    impl client::Handler for TestClient {
        type Error = anyhow::Error;
        type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;
        type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;

        fn finished_bool(self, b: bool) -> Self::FutureBool {
            futures::future::ready(Ok((self, b)))
        }

        fn finished(self, session: client::Session) -> Self::FutureUnit {
            futures::future::ready(Ok((self, session)))
        }

        fn check_server_key(self, _: &PublicKey) -> Self::FutureBool {
            self.finished_bool(true)
        }
        fn channel_open_confirmation(
            self,
            _: ChannelId,
            _: u32,
            _: u32,
            session: client::Session,
        ) -> Self::FutureUnit {
            self.finished(session)
        }
        fn data(self, _: ChannelId, _: &[u8], session: client::Session) -> Self::FutureUnit {
            self.finished(session)
        }
    }
}
