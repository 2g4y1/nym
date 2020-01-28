use crate::built_info;
use crypto::identity::{MixnetIdentityKeyPair, MixnetIdentityPrivateKey, MixnetIdentityPublicKey};
use healthcheck::HealthChecker;
use log::{error, info, trace, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock as FRwLock;
use topology::NymTopology;

const NODE_HEALTH_THRESHOLD: f64 = 0.0;

// auxiliary type for ease of use
pub type TopologyInnerRef<T> = Arc<FRwLock<Inner<T>>>;

pub(crate) struct TopologyControl<T, IDPair, Priv, Pub>
where
    T: NymTopology,
    IDPair: MixnetIdentityKeyPair<Priv, Pub>,
    Priv: MixnetIdentityPrivateKey,
    Pub: MixnetIdentityPublicKey,
{
    directory_server: String,
    inner: Arc<FRwLock<Inner<T>>>,
    health_checker: HealthChecker<IDPair, Priv, Pub>,
    refresh_rate: f64,
}

#[derive(Debug)]
enum TopologyError {
    HealthCheckError,
    NoValidPathsError,
}

impl<T, IDPair, Priv, Pub> TopologyControl<T, IDPair, Priv, Pub>
where
    T: NymTopology,
    IDPair: MixnetIdentityKeyPair<Priv, Pub>,
    Priv: MixnetIdentityPrivateKey,
    Pub: MixnetIdentityPublicKey,
{
    pub(crate) async fn new(
        directory_server: String,
        refresh_rate: f64,
        identity_keypair: IDPair,
    ) -> Self {
        // topology control run a healthcheck to determine healthy-ish nodes:
        // this is a temporary solution as the healthcheck will eventually be moved to validators

        let healthcheck_config = healthcheck::config::HealthCheck {
            directory_server: directory_server.clone(),
            // those are literally irrelevant when running single check
            interval: 100000.0,
            resolution_timeout: 5.0,
            num_test_packets: 2,
        };
        let health_checker = healthcheck::HealthChecker::new(healthcheck_config, identity_keypair);

        let mut topology_control = TopologyControl {
            directory_server,
            refresh_rate,
            inner: Arc::new(FRwLock::new(Inner::new(None))),
            health_checker,
        };

        // best effort approach to try to get a valid topology after call to 'new'
        let initial_topology = match topology_control.get_current_compatible_topology().await {
            Ok(topology) => Some(topology),
            Err(err) => {
                error!("Initial topology is invalid - {:?}. Right now it will be impossible to send any packets through the mixnet!", err);
                None
            }
        };
        topology_control
            .update_global_topology(initial_topology)
            .await;

        topology_control
    }

    async fn get_current_compatible_topology(&self) -> Result<T, TopologyError> {
        let full_topology = T::new(self.directory_server.clone());

        let healthcheck_result = self.health_checker.do_check().await;
        let healthcheck_scores = match healthcheck_result {
            Err(err) => {
                error!("Error while performing the healthcheck: {:?}", err);
                return Err(TopologyError::HealthCheckError);
            }
            Ok(scores) => scores,
        };

        let healthy_topology =
            healthcheck_scores.filter_topology_by_score(&full_topology, NODE_HEALTH_THRESHOLD);

        let versioned_healthy_topology = healthy_topology.filter_node_versions(
            built_info::PKG_VERSION,
            built_info::PKG_VERSION,
            built_info::PKG_VERSION,
        );

        // make sure you can still send a packet through the network:
        if !versioned_healthy_topology.can_construct_path_through() {
            return Err(TopologyError::NoValidPathsError);
        }

        Ok(versioned_healthy_topology)
    }

    pub(crate) fn get_inner_ref(&self) -> Arc<FRwLock<Inner<T>>> {
        self.inner.clone()
    }

    async fn update_global_topology(&mut self, new_topology: Option<T>) {
        // acquire write lock
        let mut write_lock = self.inner.write().await;
        write_lock.topology = new_topology;
    }

    pub(crate) async fn run_refresher(mut self) {
        info!("Starting topology refresher");
        let delay_duration = Duration::from_secs_f64(self.refresh_rate);
        loop {
            trace!("Refreshing the topology");
            let new_topology_res = self.get_current_compatible_topology().await;

            let new_topology = match new_topology_res {
                Ok(topology) => Some(topology),
                Err(err) => {
                    warn!("the obtained topology seems to be invalid - {:?}, it will be impossible to send packets through", err);
                    None
                }
            };

            self.update_global_topology(new_topology).await;
            tokio::time::delay_for(delay_duration).await;
        }
    }
}

pub struct Inner<T: NymTopology> {
    pub topology: Option<T>,
}

impl<T: NymTopology> Inner<T> {
    fn new(topology: Option<T>) -> Self {
        Inner { topology }
    }
}