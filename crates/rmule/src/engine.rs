use std::path::PathBuf;

use crate::configuration::{ConfigurationCommand, ConfigurationManagerHandle};

/// The rMule Engine. This contains the entire actor system that responds to
/// commands, emits events, runs downloads, updates configuration etc.
///
/// The engine is structured as a [DAG](https://en.wikipedia.org/wiki/Directed_acyclic_graph)
/// of actors (rMule calls them Managers)
/// which send commands and events to each other. Each manager is responsible
/// for a discrete task, such as handling the configuration database,
/// running searches, handling incoming chunks etc. The DAG structure is very
/// similar to the Rust data ownership tree; the fact that it works so well
/// is not a coincidence.
///
/// This architecture was inspired by this
/// [Wikipedia article on Erlang](https://en.wikipedia.org/wiki/Erlang_(programming_language)),
/// especially the part about supervisor trees. However, note that we do
/// not have a "let it crash" style, instead we are more along the lines of
/// Rust's "never crash" style. Also, there is no process isolation, instead
/// we use Tokio tasks.
///
/// The article basically describes what is also known as an
/// [Actor Model](https://en.wikipedia.org/wiki/Actor_model)
///
/// While there are Actor crates in the Rust ecosystem none of them see
/// widespread use. In accordance with rMule's principle of "that's not too
/// hard, let's do it ourselves" the engine is therefore an implementation of
/// the actor system in Rust straight over the top of Tokio. See also
/// [this blog post by Alice Ryhl](https://ryhl.io/blog/actors-with-tokio/),
/// a Tokio maintainer, which may be helpful in explaining why the handles
/// are structured as they are.
pub struct Engine {
    config_dir: PathBuf,
    cfg_mgr_handle: ConfigurationManagerHandle,
}

impl Engine {
    pub fn new<P: Into<PathBuf>>(config_dir: P) -> Self {
        let config_dir = config_dir.into();
        let cfg_mgr_handle = ConfigurationManagerHandle::new(&config_dir);

        Self {
            config_dir: config_dir.into(),
            cfg_mgr_handle,
        }
    }

    pub async fn start(&self) {
        self.cfg_mgr_handle
            .send_command(ConfigurationCommand::Start)
            .await
            .unwrap();
    }
}
